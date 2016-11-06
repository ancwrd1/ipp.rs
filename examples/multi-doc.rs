extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::consts::tag::{PRINTER_ATTRIBUTES_TAG, JOB_ATTRIBUTES_TAG};
use ipp::consts::attribute::OPERATIONS_SUPPORTED;
use ipp::consts::operation::{CREATE_JOB, SEND_DOCUMENT};
use ipp::operation::{IppOperation, GetPrinterAttributes, CreateJob, SendDocument};
use ipp::value::IppValue;

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [filename...]", args[0]);
        exit(1);
    }

    // check if printer supports create/send operations
    let mut get_op = GetPrinterAttributes::new(&args[1]);
    let printer_attrs = get_op.execute().unwrap();
    let ops_attr = printer_attrs.get(PRINTER_ATTRIBUTES_TAG, OPERATIONS_SUPPORTED).unwrap();

    if let &IppValue::ListOf(ref list) = ops_attr.value() {
        if let None = list.into_iter().find(|&e| {
            if let &IppValue::Enum(v) = e { v as u16 == CREATE_JOB || v as u16 == SEND_DOCUMENT }
            else { false }
        }) {
            println!("ERROR: target printer does not support create/send operations");
            exit(2);
        }
    }

    let mut create_op = CreateJob::new(&args[1], Some("multi-doc"));

    let job_id = create_op.execute_and_get_job_id().unwrap();
    println!("job id: {}", job_id);

    for i in 2..args.len() {
        let last = i >= (args.len() - 1);
        println!("Sending {}, last: {}", args[i], last);
        let mut f = File::open(&args[i]).unwrap();

        let mut send_op = SendDocument::new(&args[1], job_id, &mut f, &env::var("USER").unwrap(), last);
        let send_attrs = send_op.execute().unwrap();
        for (_, v) in send_attrs.get_group(JOB_ATTRIBUTES_TAG).unwrap() {
            println!("{}: {}", v.name(), v.value());
        }
    }
}
