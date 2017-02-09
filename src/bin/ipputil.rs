extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::{IppClient, IppAttribute, IppValue, PrintJob, GetPrinterAttributes, IppError};
use ipp::consts::tag::{JOB_ATTRIBUTES_TAG, PRINTER_ATTRIBUTES_TAG};

fn do_print(args: &[String]) -> Result<(), IppError> {
    let client = IppClient::new(&args[2]);

    let mut f = File::open(&args[3])?;
    let mut operation = PrintJob::new(
        &mut f,
        &env::var("USER").unwrap_or_else(|_| String::new()),
        Some(&args[1])
    );

    for arg in &args[4..] {
        let mut kv = arg.split('=');
        let (k, v) = (kv.next().unwrap_or_else(|| ""), kv.next().unwrap_or_else(|| ""));

        if k.is_empty() || v.is_empty() { continue }

        let value = if let Ok(iv) = v.parse::<i32>() {
            IppValue::Integer(iv)
        } else if v == "true" || v == "false" {
            IppValue::Boolean(v == "true")
        } else {
            IppValue::Keyword(v.to_string())
        };

        operation.add_attribute(IppAttribute::new(k, value));
    }

    let attrs = client.send(&mut operation)?;

    if let Some(group) = attrs.get_group(JOB_ATTRIBUTES_TAG) {
        for v in group.values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

fn do_info(args: &[String]) -> Result<(), IppError> {
    let client = IppClient::new(&args[2]);
    let mut operation = GetPrinterAttributes::with_attributes(&args[3..]);

    let attrs = client.send(&mut operation)?;

    if let Some(group) = attrs.get_group(PRINTER_ATTRIBUTES_TAG) {
        for v in group.values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} info uri [attr...]", args[0]);
        println!("       {} print uri filename [attr=value]", args[0]);
        exit(1);
    }

    match &args[1][..] {
        "info" => {
            if let Err(err) = do_info(&args) {
                println!("{:?}", err);
                exit(2);
            }
            
        }
        "print" => {
            if let Err(err) = do_print(&args) {
                println!("{:?}", err);
                exit(2);
            }
        }
        _ => {
            println!("ERROR: invalid operation, expected one of 'info' or 'print");
            exit(1);
        }
    }
}
