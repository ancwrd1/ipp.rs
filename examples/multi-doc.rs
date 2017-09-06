extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::{IppClient, IppValue, GetPrinterAttributes, CreateJob, SendDocument};
use ipp::consts::tag::DelimiterTag;
use ipp::consts::attribute::{OPERATIONS_SUPPORTED, JOB_ID};
use ipp::consts::operation::Operation;

fn supports_multi_doc(v: &IppValue) -> bool {
    if let IppValue::Enum(v) = *v { v == Operation::CreateJob as i32 || v == Operation::SendDocument as i32 }
    else { false }
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [filename...]", args[0]);
        exit(1);
    }

    let client = IppClient::new(&args[1]);

    // check if printer supports create/send operations
    let get_op = GetPrinterAttributes::with_attributes(&[OPERATIONS_SUPPORTED.to_string()]);
    let printer_attrs = client.send(get_op).unwrap();
    let ops_attr = printer_attrs.get(DelimiterTag::PrinterAttributes, OPERATIONS_SUPPORTED).unwrap();

    if !ops_attr.value().into_iter().any(supports_multi_doc) {
        println!("ERROR: target printer does not support create/send operations");
        exit(2);
    }

    let create_op = CreateJob::new(Some("multi-doc"));
    let attrs = client.send(create_op).unwrap();
    let job_id = match *attrs.get(DelimiterTag::JobAttributes, JOB_ID).unwrap().value() {
        IppValue::Integer(id) => id,
        _ => panic!("invalid value")
    };
    println!("job id: {}", job_id);

    for (i, item) in args.iter().enumerate().skip(2) {
        let last = i >= (args.len() - 1);
        println!("Sending {}, last: {}", item, last);
        let f = File::open(&item).unwrap();

        let send_op = SendDocument::new(job_id, Box::new(f), &env::var("USER").unwrap(), last);
        let send_attrs = client.send(send_op).unwrap();
        for v in send_attrs.get_job_attributes().unwrap().values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
}
