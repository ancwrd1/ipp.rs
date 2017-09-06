extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::{IppClient, IppAttribute, IppValue, PrintJob, GetPrinterAttributes, IppError};

fn do_print(args: &[String]) -> Result<(), IppError> {
    let f = File::open(&args[3])?;

    let client = IppClient::new(&args[2]);

    let mut operation = PrintJob::new(
        Box::new(f),
        &env::var("USER").unwrap_or_else(|_| String::new()),
        Some(&args[1])
    );

    for arg in &args[4..] {
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

fn do_status(args: &[String]) -> Result<(), IppError> {
    let client = IppClient::new(&args[2]);
    let operation = GetPrinterAttributes::with_attributes(&args[3..]);

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

fn usage(prog: &str) {
    println!("Usage: {} status uri [attr...]", prog);
    println!("       {} print uri filename [attr=value]", prog);
    println!("\nSupported uri schemes: http, ipp");
}

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        usage(&args[0]);
        exit(1);
    }

    match &args[1][..] {
        "status" => {
            if let Err(err) = do_status(&args) {
                println!("{:?}", err);
                exit(2);
            }

        }
        "print" => {
            if args.len() < 4 {
                usage(&args[0]);
                exit(1);
            }
            if let Err(err) = do_print(&args) {
                println!("{:?}", err);
                exit(2);
            }
        }
        _ => {
            println!("ERROR: invalid operation, expected one of 'status' or 'print");
            exit(1);
        }
    }
}
