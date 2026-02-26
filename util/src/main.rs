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

fn dump_attributes(response: &IppRequestResponse, tag: DelimiterTag) {
    for group in response.attributes().groups_of(tag) {
        let mut values = group.attributes().values().collect::<Vec<_>>();

        values.sort_by_key(|&a| a.name());

        for v in values {
            println!("{}: {}", v.name(), v.value());
        }
        println!();
    }
}

fn do_print_job(params: &IppParams, cmd: IppPrintCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    if !cmd.no_check_state {
        let operation = IppOperationBuilder::get_printer_attributes(client.uri().clone()).build()?;
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
            builder = builder.attribute(IppAttribute::new(k.try_into().unwrap(), v.parse().unwrap()));
        }
    }

    let response = client.send(builder.build()?)?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    dump_attributes(&response, DelimiterTag::JobAttributes);

    Ok(())
}

fn do_status(params: &IppParams, cmd: IppStatusCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    let operation = IppOperationBuilder::get_printer_attributes(client.uri().clone())
        .attributes(&cmd.attributes)
        .build()?;

    let response = client.send(operation)?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    dump_attributes(&response, DelimiterTag::PrinterAttributes);

    Ok(())
}

fn do_purge_jobs(params: &IppParams, cmd: IppPurgeCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    let mut builder = IppOperationBuilder::purge_jobs(client.uri().clone());

    if let Some(username) = cmd.user_name {
        builder = builder.user_name(username);
    }

    let operation = builder.build()?;

    let response = client.send(operation)?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    dump_attributes(&response, DelimiterTag::OperationAttributes);

    Ok(())
}

fn do_cancel_job(params: &IppParams, cmd: IppCancelCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    let mut builder = IppOperationBuilder::cancel_job(client.uri().clone(), cmd.job_id);

    if let Some(username) = cmd.user_name {
        builder = builder.user_name(username);
    }

    let operation = builder.build()?;

    let response = client.send(operation)?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    dump_attributes(&response, DelimiterTag::OperationAttributes);

    Ok(())
}

fn do_get_job(params: &IppParams, cmd: IppGetJobCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    let mut builder = IppOperationBuilder::get_job_attributes(client.uri().clone(), cmd.job_id);

    if let Some(username) = cmd.user_name {
        builder = builder.user_name(username);
    }

    let operation = builder.build()?;

    let response = client.send(operation)?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    dump_attributes(&response, DelimiterTag::JobAttributes);

    Ok(())
}

fn do_get_all_jobs(params: &IppParams, cmd: IppGetAllJobsCmd) -> Result<(), IppError> {
    let client = new_client(cmd.uri.parse()?, params)?;

    let mut builder = IppOperationBuilder::get_jobs(client.uri().clone());

    if let Some(username) = cmd.user_name {
        builder = builder.user_name(username);
    }

    let operation = builder.build()?;

    let response = client.send(operation)?;

    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    dump_attributes(&response, DelimiterTag::JobAttributes);

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
    PrintJob(IppPrintCmd),
    #[clap(name = "status", about = "Get status of an IPP printer")]
    Status(IppStatusCmd),
    #[clap(name = "cancel-job", about = "Cancel job from an IPP printer")]
    CancelJob(IppCancelCmd),
    #[clap(name = "get-job", about = "Get job attributes from an IPP printer")]
    GetJob(IppGetJobCmd),
    #[clap(name = "purge-jobs", about = "Purge all jobs from an IPP printer")]
    PurgeJobs(IppPurgeCmd),
    #[clap(name = "get-all-jobs", about = "Get pending jobs from an IPP printer")]
    GetAllJobs(IppGetAllJobsCmd),
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

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
struct IppPurgeCmd {
    #[clap(help = "Printer URI")]
    uri: String,
    #[clap(
        long = "user-name",
        short = 'u',
        help = "User name to send as requesting-user-name attribute"
    )]
    user_name: Option<String>,
}

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
struct IppCancelCmd {
    #[clap(help = "Printer URI")]
    uri: String,
    #[clap(long = "job-id", short = 'j', help = "Job ID to cancel")]
    job_id: i32,
    #[clap(
        long = "user-name",
        short = 'u',
        help = "User name to send as requesting-user-name attribute"
    )]
    user_name: Option<String>,
}

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
struct IppGetJobCmd {
    #[clap(help = "Printer URI")]
    uri: String,
    #[clap(long = "job-id", short = 'j', help = "Job ID to get attributes for")]
    job_id: i32,
    #[clap(
        long = "user-name",
        short = 'u',
        help = "User name to send as requesting-user-name attribute"
    )]
    user_name: Option<String>,
}

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
struct IppGetAllJobsCmd {
    #[clap(help = "Printer URI")]
    uri: String,
    #[clap(
        long = "user-name",
        short = 'u',
        help = "User name to send as requesting-user-name attribute"
    )]
    user_name: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = IppParams::parse();

    match params.command {
        IppCommand::Status(ref cmd) => do_status(&params, cmd.clone())?,
        IppCommand::PrintJob(ref cmd) => do_print_job(&params, cmd.clone())?,
        IppCommand::CancelJob(ref cmd) => do_cancel_job(&params, cmd.clone())?,
        IppCommand::GetJob(ref cmd) => do_get_job(&params, cmd.clone())?,
        IppCommand::PurgeJobs(ref cmd) => do_purge_jobs(&params, cmd.clone())?,
        IppCommand::GetAllJobs(ref cmd) => do_get_all_jobs(&params, cmd.clone())?,
    }
    Ok(())
}
