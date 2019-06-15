//!
//! High-level utility functions to be used from external application or command-line utility
//!

use std::{ffi::OsString, io};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand, Values};
use log::debug;
use num_traits::FromPrimitive;

use futures::{future, Future};
use ipp_client::{IppClient, IppClientBuilder, IppError};
use ipp_proto::{
    attribute::{PRINTER_STATE, PRINTER_STATE_REASONS},
    ipp::{DelimiterTag, PrinterState},
    operation::{GetPrinterAttributes, PrintJob},
    IppAttribute, IppReadStream, IppValue,
};
use tokio::io::AsyncRead;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const ERROR_STATES: &[&str] = &[
    "media-jam",
    "toner-empty",
    "spool-area-full",
    "cover-open",
    "door-open",
    "input-tray-missing",
    "output-tray-missing",
    "marker-supply-empty",
    "paused",
    "shutdown",
];

fn unwrap_values(values: Option<Values>) -> Values {
    values.unwrap_or_else(Values::default)
}

fn new_client(matches: &ArgMatches) -> IppClient {
    let mut builder = IppClientBuilder::new(matches.value_of("uri").unwrap())
        .timeout(matches.value_of("timeout").unwrap().parse::<u64>().unwrap())
        .ca_certs(&unwrap_values(matches.values_of("cacert")).collect::<Vec<_>>());

    if matches.is_present("noverifyhostname") {
        builder = builder.verify_hostname(false);
    }

    if matches.is_present("noverifycertificate") {
        builder = builder.verify_certificate(false);
    }

    builder.build()
}

fn is_status_ok(matches: &ArgMatches) -> Result<(), IppError> {
    if !matches.is_present("nocheckstate") {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let client = new_client(matches);
        let operation = GetPrinterAttributes::with_attributes(&[PRINTER_STATE, PRINTER_STATE_REASONS]);
        let attrs = runtime.block_on(client.send(operation))?;

        if let Some(a) = attrs.get(DelimiterTag::PrinterAttributes, PRINTER_STATE) {
            if let IppValue::Enum(ref e) = *a.value() {
                if let Some(state) = PrinterState::from_i32(*e) {
                    if state == PrinterState::Stopped {
                        debug!("Printer is stopped");
                        return Err(IppError::PrinterStateError(vec!["stopped".to_string()]));
                    }
                }
            }
        }

        if let Some(reasons) = attrs.get(DelimiterTag::PrinterAttributes, PRINTER_STATE_REASONS) {
            let keywords = match *reasons.value() {
                IppValue::ListOf(ref v) => v
                    .iter()
                    .filter_map(|e| {
                        if let IppValue::Keyword(ref k) = *e {
                            Some(k.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                IppValue::Keyword(ref v) => vec![v.clone()],
                _ => Vec::new(),
            };
            if keywords.iter().any(|k| ERROR_STATES.contains(&&k[..])) {
                debug!("Printer is in error state: {:?}", keywords);
                return Err(IppError::PrinterStateError(keywords.clone()));
            }
        }
    }
    Ok(())
}

struct FileSource {
    inner: Box<AsyncRead>,
}
unsafe impl Send for FileSource {}

fn make_operation(matches: &ArgMatches) -> Box<dyn Future<Item = FileSource, Error = io::Error> + Send + 'static> {
    match matches.value_of("filename").map(|v| v.to_owned()) {
        Some(filename) => {
            Box::new(tokio::fs::File::open(filename).and_then(|file| Ok(FileSource { inner: Box::new(file) })))
        }
        None => Box::new(future::ok(FileSource {
            inner: Box::new(tokio::io::stdin()),
        })),
    }
}

fn do_print(matches: &ArgMatches) -> Result<(), IppError> {
    let _ = is_status_ok(matches)?;

    let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let client = new_client(matches);

    let username = matches
        .value_of("username")
        .map(|v| v.to_owned())
        .unwrap_or_else(String::new);

    let jobname = matches.value_of("jobname").map(|v| v.to_owned());

    let options = unwrap_values(matches.values_of("option"))
        .map(|v| v.to_owned())
        .collect::<Vec<String>>();

    let fut = make_operation(&matches);

    runtime.block_on(fut.map_err(IppError::from).and_then(move |source| {
        let mut operation = PrintJob::new(IppReadStream::new(source.inner), username, jobname);

        for arg in options {
            let mut kv = arg.split('=');
            if let Some(k) = kv.next() {
                if let Some(v) = kv.next() {
                    let value = if let Ok(iv) = v.parse::<i32>() {
                        IppValue::Integer(iv)
                    } else if v == "true" || v == "false" {
                        IppValue::Boolean(v == "true")
                    } else {
                        IppValue::Keyword(v.to_string())
                    };
                    operation.add_attribute(IppAttribute::new(k, value));
                }
            }
        }

        client.send(operation).and_then(|attrs| {
            if let Some(group) = attrs.job_attributes() {
                for v in group.values() {
                    println!("{}: {}", v.name(), v.value());
                }
            }
            Ok(())
        })
    }))
}

fn do_status(matches: &ArgMatches) -> Result<(), IppError> {
    let client = new_client(matches);

    let operation =
        GetPrinterAttributes::with_attributes(&unwrap_values(matches.values_of("attribute")).collect::<Vec<_>>());

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let attrs = runtime.block_on(client.send(operation))?;

    if let Some(group) = attrs.printer_attributes() {
        let mut values: Vec<_> = group.values().collect();
        values.sort_by(|a, b| a.name().cmp(b.name()));
        for v in values {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

/// Entry point to main utility function
///
/// * `args` - a list of arguments including program name as a first argument
///
/// Command line usage for getting printer status (will print list of printer attributes on stdout)
/// ```text
/// USAGE:
/// ipputil status [FLAGS] [OPTIONS] <uri>
///
/// FLAGS:
///     -h, --help                     Prints help information
///     --no-verify-certificate    Disable server certificate verification
///     --no-verify-hostname       Disable server host name verification
///
/// OPTIONS:
///     -a, --attribute <attribute>...    IPP attribute to query, default is get all
///     -c, --cacert <filename>...        Additional root certificates in PEM or DER format
///     -t, --timeout <timeout>           Network timeout in seconds, 0 to disable [default: 30]
///
/// ARGS:
///     <uri>    Printer URI, supported schemes: ipp, ipps, http, https
///```
/// Command line usage for printing the document
/// ```text
/// USAGE:
///     ipputil print [FLAGS] [OPTIONS] <uri>
///
/// FLAGS:
///     -h, --help                     Prints help information
///     -n, --no-check-state           Do not check printer state before printing
///         --no-verify-certificate    Disable server certificate verification
///         --no-verify-hostname       Disable server host name verification
///
/// OPTIONS:
///     -c, --cacert <filename>...     Additional root certificates in PEM or DER format
///     -f, --file <filename>          Input file name to print [default: standard input]
///     -j, --job <jobname>            Job name to send as job-name attribute
///     -o, --option <key=value>...    Extra IPP job attributes to send
///     -t, --timeout <timeout>        Network timeout in seconds, 0 to disable [default: 30]
///     -u, --user <username>          User name to send as requesting-user-name attribute
///
/// ARGS:
///     <uri>    Printer URI, supported schemes: ipp, ipps, http, https
/// ```
pub fn util_main<I, T>(args: I) -> Result<(), IppError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = App::new("IPP utility")
        .version(VERSION)
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("cacert")
                .short("c")
                .long("cacert")
                .value_name("filename")
                .multiple(true)
                .number_of_values(1)
                .help("Additional root certificates in PEM or DER format")
                .global(true)
                .required(false),
        )
        .arg(
            Arg::with_name("noverifyhostname")
                .long("--no-verify-hostname")
                .help("Disable server host name verification")
                .global(true)
                .required(false),
        )
        .arg(
            Arg::with_name("noverifycertificate")
                .long("--no-verify-certificate")
                .help("Disable server certificate verification")
                .global(true)
                .required(false),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("--timeout")
                .help("Network timeout in seconds, 0 to disable")
                .global(true)
                .default_value("30"),
        )
        .subcommand(
            SubCommand::with_name("print")
                .about("Print file to an IPP printer")
                .arg(
                    Arg::with_name("nocheckstate")
                        .short("n")
                        .long("no-check-state")
                        .help("Do not check printer state before printing")
                        .required(false),
                )
                .arg(
                    Arg::with_name("filename")
                        .short("f")
                        .long("file")
                        .value_name("filename")
                        .help("Input file name to print [default: standard input]")
                        .required(false),
                )
                .arg(
                    Arg::with_name("username")
                        .short("u")
                        .long("user")
                        .value_name("username")
                        .help("User name to send as requesting-user-name attribute")
                        .required(false),
                )
                .arg(
                    Arg::with_name("jobname")
                        .short("j")
                        .long("job")
                        .value_name("jobname")
                        .help("Job name to send as job-name attribute")
                        .required(false),
                )
                .arg(
                    Arg::with_name("option")
                        .short("o")
                        .long("option")
                        .value_name("key=value")
                        .help("Extra IPP job attributes to send")
                        .multiple(true)
                        .number_of_values(1)
                        .required(false),
                )
                .arg(
                    Arg::with_name("uri")
                        .index(1)
                        .value_name("uri")
                        .required(true)
                        .help("Printer URI, supported schemes: ipp, ipps, http, https"),
                ),
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Get status of an IPP printer")
                .arg(
                    Arg::with_name("attribute")
                        .short("a")
                        .long("attribute")
                        .value_name("attribute")
                        .multiple(true)
                        .number_of_values(1)
                        .required(false)
                        .help("IPP attribute to query, default is get all"),
                )
                .arg(
                    Arg::with_name("uri")
                        .index(1)
                        .value_name("uri")
                        .required(true)
                        .help("Printer URI, supported schemes: ipp, ipps, http, https"),
                ),
        )
        .get_matches_from_safe(args)
        .map_err(|e| IppError::ParamError(e.to_string()))?;

    if let Some(printcmd) = args.subcommand_matches("print") {
        do_print(printcmd)
    } else if let Some(statuscmd) = args.subcommand_matches("status") {
        do_status(statuscmd)
    } else {
        panic!("Fatal argument error");
    }
}
