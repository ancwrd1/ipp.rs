//!
//! High-level utility functions to be used from external application or command-line utility
//!

use std::{ffi::OsString, io, path::PathBuf};

use futures::AsyncRead;
use structopt::StructOpt;

use ipp_client::{IppClient, IppClientBuilder, IppError};
use ipp_proto::{ipp::DelimiterTag, IppAttribute, IppOperationBuilder, IppValue};

fn new_client(uri: &str, params: &IppParams) -> IppClient {
    IppClientBuilder::new(&uri)
        .timeout(params.timeout)
        .ignore_tls_errors(params.ignore_tls_errors)
        .build()
}

async fn new_reader(cmd: &IppPrintCmd) -> io::Result<Box<dyn AsyncRead + Send + Unpin>> {
    let file: Box<dyn AsyncRead + Send + Unpin> = match cmd.file {
        Some(ref filename) => {
            let file = async_std::fs::File::open(filename).await?;
            Box::new(file)
        }
        None => Box::new(async_std::io::stdin()),
    };
    Ok(file)
}

async fn do_print(params: &IppParams, cmd: IppPrintCmd) -> Result<(), IppError> {
    let client = new_client(&cmd.uri, params);

    if !cmd.no_check_state {
        client.check_ready().await?;
    }

    let reader = new_reader(&cmd).await.map_err(IppError::from)?;

    let mut builder = IppOperationBuilder::print_job(reader);
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

    let attrs = client.send(builder.build()).await?;
    if let Some(group) = attrs.groups_of(DelimiterTag::JobAttributes).get(0) {
        for v in group.attributes().values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

async fn do_status(params: &IppParams, cmd: IppStatusCmd) -> Result<(), IppError> {
    let client = new_client(&cmd.uri, &params);

    let operation = IppOperationBuilder::get_printer_attributes()
        .attributes(&cmd.attributes)
        .build();

    let attrs = client.send(operation).await?;

    if let Some(group) = attrs.groups_of(DelimiterTag::PrinterAttributes).get(0) {
        let mut values: Vec<_> = group.attributes().values().collect();
        values.sort_by(|a, b| a.name().cmp(b.name()));
        for v in values {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

#[derive(StructOpt)]
#[structopt(about = "IPP print utility", name = "ipputil", rename_all = "kebab-case")]
struct IppParams {
    #[structopt(
        long = "ignore-tls-errors",
        short = "i",
        global = true,
        help = "Ignore TLS handshake errors"
    )]
    ignore_tls_errors: bool,

    #[structopt(
        default_value = "0",
        long = "timeout",
        short = "t",
        global = true,
        help = "Connect timeout in seconds, 0 = no timeout"
    )]
    timeout: u64,

    #[structopt(subcommand)]
    command: IppCommand,
}

#[derive(StructOpt)]
enum IppCommand {
    #[structopt(name = "print", about = "Print file to an IPP printer")]
    Print(IppPrintCmd),
    #[structopt(name = "status", about = "Get status of an IPP printer")]
    Status(IppStatusCmd),
}

#[derive(StructOpt, Clone)]
#[structopt(rename_all = "kebab-case")]
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
#[structopt(rename_all = "kebab-case")]
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
///     -i, --ignore-tls-errors        Ignore TLS handshake errors
///     -V, --version                  Prints version information
///
/// OPTIONS:
///     -a, --attribute <attributes>...   Attributes to query, default is to get all
///     -t, --timeout <timeout>           Connect timeout in seconds, 0 to disable [default: 30]
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
///     -i, --ignore-tls-errors        Ignore TLS handshake errors
///     -n, --no-check-state           Do not check printer state before printing
///     -V, --version                  Prints version information
///
/// OPTIONS:
///     -f, --file <file>              Input file name to print [default: standard input]
///     -j, --job-name <job-name>      Job name to send as job-name attribute
///     -o, --option <options>...      Extra IPP job attributes in key=value format
///     -t, --timeout <timeout>        Connect timeout in seconds, 0 to disable [default: 30]
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
        IppCommand::Status(ref cmd) => async_std::task::block_on(do_status(&params, cmd.clone()))?,
        IppCommand::Print(ref cmd) => async_std::task::block_on(do_print(&params, cmd.clone()))?,
    }
    Ok(())
}
