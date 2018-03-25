use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{stdin, Read};

use num_traits::FromPrimitive;
use clap::{Arg, ArgMatches, App, AppSettings, SubCommand, Values};

use ::{IppClient, IppAttribute, IppValue, PrintJob, GetPrinterAttributes, IppError};
use consts::tag::DelimiterTag;
use consts::attribute::{PrinterState, PRINTER_STATE, PRINTER_STATE_REASONS};

const GIT_VERSION: &str = env!("GIT_VERSION");

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
    "shutdown"];

fn unwrap_values(values: Option<Values>) -> Values {
    values.unwrap_or_else(|| Values::default())
}

fn new_client(matches: &ArgMatches) -> IppClient {
    IppClient::with_root_certificates(
        matches.value_of("uri").unwrap(),
        &unwrap_values(matches.values_of("cacert")).collect::<Vec<_>>())
}

fn do_print(matches: &ArgMatches) -> Result<(), IppError> {
    let reader: Box<Read> = match matches.value_of("filename") {
        Some(filename) => Box::new(File::open(filename)?),
        None => Box::new(stdin())
    };

    let operation = GetPrinterAttributes::with_attributes(&[PRINTER_STATE, PRINTER_STATE_REASONS]);

    let client = new_client(matches);

    if !matches.is_present("nocheckstate") {
        let attrs = client.send(operation)?;

        if let Some(&ref a) = attrs.get(DelimiterTag::PrinterAttributes, PRINTER_STATE) {
            if let &IppValue::Enum(ref e) = a.value() {
                if let Some(state) = PrinterState::from_i32(*e) {
                    if state == PrinterState::Stopped {
                        debug!("Printer is stopped");
                        return Err(IppError::PrinterStateError(vec!["stopped".to_string()]));
                    }
                }
            }
        }

        if let Some(&ref reasons) = attrs.get(DelimiterTag::PrinterAttributes, PRINTER_STATE_REASONS) {
            let keywords = match reasons.value() {
                &IppValue::ListOf(ref v) =>
                    v.iter().filter_map(|e| if let &IppValue::Keyword(ref k) = e { Some(k.clone()) } else { None }).collect(),
                &IppValue::Keyword(ref v) => vec![v.clone()],
                _ => Vec::new()
            };
            if keywords.iter().any(|k| ERROR_STATES.contains(&&k[..])) {
                debug!("Printer is in error state: {:?}", keywords);
                return Err(IppError::PrinterStateError(keywords.clone()));
            }
        }
    }

    let mut operation = PrintJob::new(
        reader,
        matches.value_of("username").unwrap_or(&env::var("USER").unwrap_or_else(|_| String::new())),
        matches.value_of("jobname")
    );

    for arg in unwrap_values(matches.values_of("option")) {
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

    let attrs = client.send(operation)?;

    if let Some(group) = attrs.get_job_attributes() {
        for v in group.values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

fn do_status(matches: &ArgMatches) -> Result<(), IppError> {
    let client = new_client(matches);

    let operation = GetPrinterAttributes::with_attributes(
        &unwrap_values(matches.values_of("attribute")).collect::<Vec<_>>());

    let attrs = client.send(operation)?;

    if let Some(group) = attrs.get_printer_attributes() {
        let mut values: Vec<_> = group.values().collect();
        values.sort_by(|&a, &b| a.name().cmp(b.name()));
        for v in &values {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

pub fn util_main<'a, I, T>(args: I) -> Result<(), IppError>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone {
    let args = App::new("IPP utility")
        .version(GIT_VERSION)
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(SubCommand::with_name("print")
            .about("Print file to an IPP printer")
            .arg(Arg::with_name("cacert")
                .short("c")
                .long("cacert")
                .value_name("filename")
                .multiple(true)
                .number_of_values(1)
                .help("Additional root certificate in DER format")
                .required(false))
            .arg(Arg::with_name("nocheckstate")
                .short("n")
                .long("no-check-state")
                .help("Do not check printer state before printing")
                .required(false))
            .arg(Arg::with_name("filename")
                .short("f")
                .long("file")
                .value_name("filename")
                .help("Input file name to print. If missing will read from stdin")
                .required(false))
            .arg(Arg::with_name("username")
                .short("u")
                .long("user")
                .value_name("username")
                .help("User name to send as requesting-user-name attribute")
                .required(false))
            .arg(Arg::with_name("jobname")
                .short("j")
                .long("job")
                .value_name("jobname")
                .help("Job name to send as job-name attribute")
                .required(false))
            .arg(Arg::with_name("option")
                .short("o")
                .long("option")
                .value_name("key=value")
                .help("Extra IPP job attributes to send")
                .multiple(true)
                .number_of_values(1)
                .required(false))
            .arg(Arg::with_name("uri")
                .index(1)
                .value_name("uri")
                .required(true)
                .help("URI to print to, supported schemes: ipp, ipps, http, https")))
        .subcommand(SubCommand::with_name("status")
            .about("Get status of an IPP printer")
            .arg(Arg::with_name("cacert")
                .short("c")
                .long("cacert")
                .value_name("filename")
                .multiple(true)
                .number_of_values(1)
                .help("Additional root certificate in DER format")
                .required(false))
            .arg(Arg::with_name("attribute")
                .short("a")
                .long("attribute")
                .value_name("attribute")
                .multiple(true)
                .number_of_values(1)
                .required(false)
                .help("IPP attribute to query, default is get all"))
            .arg(Arg::with_name("uri")
                .index(1)
                .value_name("uri")
                .required(true)
                .help("URI to print to, supported schemes: ipp, ipps, http, https")))
        .get_matches_from_safe(args)?;

    if let Some(printcmd) = args.subcommand_matches("print") {
        do_print(printcmd)
    } else if let Some(statuscmd) = args.subcommand_matches("status") {
        do_status(statuscmd)
    } else {
        panic!("Fatal argument error");
    }
}
