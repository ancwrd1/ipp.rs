//!
//! High-level utility functions to be used from external application or command-line utility
//!

use std::{ffi::OsString, io, path::PathBuf};

use futures::{future, Future};
use structopt::StructOpt;
use tokio::io::AsyncRead;

use ipp_client::{IppClient, IppClientBuilder, IppError};
use ipp_proto::{IppAttribute, IppOperationBuilder, IppValue};

fn new_client(uri: &str, params: &IppParams) -> IppClient {
    IppClientBuilder::new(&uri)
        .timeout(params.timeout)
        .ca_certs(&params.ca_certs)
        .verify_hostname(!params.no_verify_hostname)
        .verify_certificate(!params.no_verify_certificate)
        .build()
}

struct FileSource {
    inner: Box<AsyncRead + Send>,
}

fn new_source(cmd: &IppPrintCmd) -> Box<dyn Future<Item = FileSource, Error = io::Error> + Send + 'static> {
    match cmd.file {
        Some(ref filename) => Box::new(
            tokio::fs::File::open(filename.to_owned()).and_then(|file| Ok(FileSource { inner: Box::new(file) })),
        ),
        None => Box::new(future::ok(FileSource {
            inner: Box::new(tokio::io::stdin()),
        })),
    }
}

fn do_print(params: &IppParams, cmd: IppPrintCmd) -> Result<(), IppError> {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let client = new_client(&cmd.uri, params);

    if !cmd.no_check_state {
        let _ = runtime.block_on(client.check_ready())?;
    }

    runtime.block_on(new_source(&cmd).map_err(IppError::from).and_then(move |source| {
        let mut builder = IppOperationBuilder::print_job(source.inner);
        if let Some(jobname) = cmd.job_name {
            builder = builder.job_title(&jobname);
        }
        if let Some(username) = cmd.user_name {
            builder = builder.user_name(&username);
        }

        for arg in cmd.options {
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
                    builder = builder.attribute(IppAttribute::new(k, value));
                }
            }
        }

        client.send(builder.build()).and_then(|attrs| {
            if let Some(group) = attrs.job_attributes() {
                for v in group.values() {
                    println!("{}: {}", v.name(), v.value());
                }
            }
            Ok(())
        })
    }))
}

fn do_status(params: &IppParams, cmd: IppStatusCmd) -> Result<(), IppError> {
    let client = new_client(&cmd.uri, &params);

    let operation = IppOperationBuilder::get_printer_attributes()
        .attributes(&cmd.attributes)
        .build();

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

#[derive(StructOpt)]
#[structopt(name = "IPP print utility", about = "", author = "", rename_all = "kebab-case")]
struct IppParams {
    #[structopt(
        long = "ca-cert",
        short = "c",
        global = true,
        help = "Additional CA root certificates in PEM or DER format"
    )]
    ca_certs: Vec<String>,

    #[structopt(
        long = "no-verify-hostname",
        global = true,
        help = "Disable TLS host name verification (insecure!)"
    )]
    no_verify_hostname: bool,

    #[structopt(
        long = "no-verify-certificate",
        global = true,
        help = "Disable TLS certificate verification (insecure!)"
    )]
    no_verify_certificate: bool,

    #[structopt(
        default_value = "0",
        long = "timeout",
        short = "t",
        global = true,
        help = "IPP request timeout in seconds, 0 = no timeout"
    )]
    timeout: u64,

    #[structopt(subcommand)]
    command: IppCommand,
}

#[derive(StructOpt)]
enum IppCommand {
    #[structopt(name = "print", about = "Print file to an IPP printer", author = "")]
    Print(IppPrintCmd),
    #[structopt(name = "status", about = "Get status of an IPP printer", author = "")]
    Status(IppStatusCmd),
}

#[derive(StructOpt, Clone)]
#[structopt(name = "IPP print utility", about = "", author = "", rename_all = "kebab-case")]
struct IppPrintCmd {
    #[structopt(help = "Printer URI")]
    uri: String,

    #[structopt(
        long = "no-check-state",
        short = "n",
        help = "Do not check printer state before printing"
    )]
    no_check_state: bool,

    #[structopt(
        long = "file",
        short = "f",
        help = "Input file name to print [default: standard input]"
    )]
    file: Option<PathBuf>,

    #[structopt(long = "job-name", short = "j", help = "Job name to send as job-name attribute")]
    job_name: Option<String>,

    #[structopt(
        long = "user-name",
        short = "u",
        help = "User name to send as requesting-user-name attribute"
    )]
    user_name: Option<String>,

    #[structopt(long = "option", short = "o", help = "Extra IPP job attributes in key=value format")]
    options: Vec<String>,
}

#[derive(StructOpt, Clone)]
#[structopt(name = "IPP print utility", about = "", author = "", rename_all = "kebab-case")]
struct IppStatusCmd {
    #[structopt(help = "Printer URI")]
    uri: String,

    #[structopt(long = "attribute", short = "a", help = "Attributes to query, default is to get all")]
    attributes: Vec<String>,
}

/// Entry point to main utility function
///
/// * `args` - a list of arguments including program name as a first argument
///
/// Command line usage for getting printer status (will print list of printer attributes on stdout)
/// ```text
/// USAGE:
///     ipputil status [FLAGS] [OPTIONS] <uri>
///
/// FLAGS:
///     -h, --help                     Prints help information
///     --no-verify-certificate        Disable TLS certificate verification (insecure)
///     --no-verify-hostname           Disable TLS host name verification (insecure)
///     -V, --version                  Prints version information
///
/// OPTIONS:
///     -a, --attribute <attributes>...   Attributes to query, default is to get all
///     -c, --ca-cert <ca-certs>...       Additional CA root certificates in PEM or DER format
///     -t, --timeout <timeout>           Network timeout in seconds, 0 to disable [default: 30]
///
/// ARGS:
///     <uri>    Printer URI
/// ```
///
/// Command line usage for getting printer status (will print list of printer attributes on stdout)
/// ```text
/// USAGE:
///     ipputil print [FLAGS] [OPTIONS] <uri>
///
/// FLAGS:
///     -h, --help                     Prints help information
///     -n, --no-check-state           Do not check printer state before printing
///     --no-verify-certificate        Disable TLS certificate verification (insecure)
///     --no-verify-hostname           Disable TLS host name verification (insecure)
///     -V, --version                  Prints version information
///
/// OPTIONS:
///     -c, --ca-cert <ca-certs>...    Additional CA root certificates in PEM or DER format
///     -f, --file <file>              Input file name to print [default: standard input]
///     -j, --job-name <job-name>      Job name to send as job-name attribute
///     -o, --option <options>...      Extra IPP job attributes in key=value format
///     -t, --timeout <timeout>        Network timeout in seconds, 0 to disable [default: 30]
///     -u, --user-name <user-name>    User name to send as requesting-user-name attribute
///
/// ARGS:
///     <uri>    Printer URI
/// ```
pub fn ipp_main<I, T>(args: I) -> Result<(), IppError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let params = IppParams::from_iter_safe(args).map_err(|e| IppError::ParamError(e.to_string()))?;
    match params.command {
        IppCommand::Status(ref cmd) => do_status(&params, cmd.clone())?,
        IppCommand::Print(ref cmd) => do_print(&params, cmd.clone())?,
    }
    Ok(())
}
