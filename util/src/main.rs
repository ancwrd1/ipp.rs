//!
//! Command-line IPP utility to print a document or get printer status
//!

#![allow(clippy::result_large_err)]

use std::{
    fs,
    io::{self, BufReader},
    path::PathBuf,
    time::Duration,
};

use clap::Parser;

use ipp::{prelude::*, util};

fn new_client(uri: Uri, params: &IppParams) -> io::Result<IppClient> {
    let mut builder = IppClient::builder(uri).ignore_tls_errors(params.ignore_tls_errors);
    if let Some(timeout) = params.timeout {
        builder = builder.request_timeout(Duration::from_secs(timeout));
    }

    for param in &params.headers {
        if let Some((k, v)) = param.split_once('=') {
            builder = builder.http_header(k, v);
        }
    }

    for cert in &params.ca_certs {
        builder = builder.ca_cert(fs::read(cert)?);
    }

    Ok(builder.build())
}

fn new_payload(cmd: &IppPrintCmd) -> io::Result<IppPayload> {
    let payload = match cmd.file {
        Some(ref filename) => IppPayload::new(BufReader::new(fs::File::open(filename)?)),
        None => IppPayload::new(BufReader::new(io::stdin())),
    };
    Ok(payload)
}

fn do_print(params: &IppParams, cmd: IppPrintCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    if !cmd.no_check_state {
        let operation = IppOperationBuilder::get_printer_attributes(client.uri().clone()).build();
        let response = client.send(operation)?;
        if !util::is_printer_ready(&response)? {
            return Err(IppError::PrinterNotReady);
        }
    }

    let payload = new_payload(&cmd).map_err(IppError::from)?;

    let mut builder = IppOperationBuilder::print_job(client.uri().clone(), payload);
    if let Some(jobname) = cmd.job_name {
        builder = builder.job_title(jobname);
    }
    if let Some(username) = cmd.user_name {
        builder = builder.user_name(username);
    }

    for arg in cmd.options {
        if let Some((k, v)) = arg.split_once('=') {
            builder = builder.attribute(IppAttribute::new(k, v.parse().unwrap()));
        }
    }

    let response = client.send(builder.build())?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    if let Some(group) = response.attributes().groups_of(DelimiterTag::JobAttributes).next() {
        for v in group.attributes().values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

fn do_status(params: &IppParams, cmd: IppStatusCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    let operation = IppOperationBuilder::get_printer_attributes(client.uri().clone())
        .attributes(&cmd.attributes)
        .build();

    let response = client.send(operation)?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    let mut values = response
        .attributes()
        .groups_of(DelimiterTag::PrinterAttributes)
        .flat_map(|group| group.attributes().values())
        .collect::<Vec<_>>();

    values.sort_by_key(|&a| a.name());

    for v in values {
        println!("{}: {}", v.name(), v.value());
    }

    Ok(())
}

#[derive(Parser)]
#[clap(about = "IPP print utility", name = "ipputil", rename_all = "kebab-case")]
struct IppParams {
    #[clap(
        long = "ignore-tls-errors",
        short = 'i',
        global = true,
        help = "Ignore TLS handshake errors"
    )]
    ignore_tls_errors: bool,

    #[clap(
        long = "ca-cert",
        short = 'c',
        global = true,
        help = "One or more additional CA certs in PEM or DER format"
    )]
    ca_certs: Vec<PathBuf>,

    #[clap(
        long = "timeout",
        short = 't',
        global = true,
        help = "Request timeout in seconds, default = no timeout"
    )]
    timeout: Option<u64>,

    #[clap(long = "header", short = 'H', help = "Extra HTTP headers in key=value format")]
    headers: Vec<String>,

    #[clap(subcommand)]
    command: IppCommand,
}

#[derive(Parser)]
enum IppCommand {
    #[clap(name = "print", about = "Print file to an IPP printer")]
    Print(IppPrintCmd),
    #[clap(name = "status", about = "Get status of an IPP printer")]
    Status(IppStatusCmd),
}

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
struct IppPrintCmd {
    #[clap(help = "Printer URI")]
    uri: String,

    #[clap(
        long = "no-check-state",
        short = 'n',
        help = "Do not check printer state before printing"
    )]
    no_check_state: bool,

    #[clap(
        long = "file",
        short = 'f',
        help = "Input file name to print [default: standard input]"
    )]
    file: Option<PathBuf>,

    #[clap(long = "job-name", short = 'j', help = "Job name to send as job-name attribute")]
    job_name: Option<String>,

    #[clap(
        long = "user-name",
        short = 'u',
        help = "User name to send as requesting-user-name attribute"
    )]
    user_name: Option<String>,

    #[clap(long = "option", short = 'o', help = "Extra IPP job attributes in key=value format")]
    options: Vec<String>,
}

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
struct IppStatusCmd {
    #[clap(help = "Printer URI")]
    uri: String,

    #[clap(long = "attribute", short = 'a', help = "Attributes to query, default is to get all")]
    attributes: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = IppParams::parse();

    match params.command {
        IppCommand::Status(ref cmd) => do_status(&params, cmd.clone())?,
        IppCommand::Print(ref cmd) => do_print(&params, cmd.clone())?,
    }
    Ok(())
}
