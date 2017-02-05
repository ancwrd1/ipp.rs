extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::{IppClient, IppValue, GetPrinterAttributes, CreateJob, SendDocument};
use ipp::consts::tag::{PRINTER_ATTRIBUTES_TAG, JOB_ATTRIBUTES_TAG};
use ipp::consts::attribute::{OPERATIONS_SUPPORTED, JOB_ID};
use ipp::consts::operation::{CREATE_JOB, SEND_DOCUMENT};

fn supports_multi_doc(v: IppValue) -> bool {
    if let IppValue::Enum(v) = v { v as u16 == CREATE_JOB || v as u16 == SEND_DOCUMENT }
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
    let mut get_op = GetPrinterAttributes::with_attributes(&[OPERATIONS_SUPPORTED.to_string()]);
    let printer_attrs = client.send(&mut get_op).unwrap();
    let ops_attr = printer_attrs.get(PRINTER_ATTRIBUTES_TAG, OPERATIONS_SUPPORTED).unwrap();

    if !ops_attr.value().clone().into_iter().any(supports_multi_doc) {
        println!("ERROR: target printer does not support create/send operations");
        exit(2);
    }

    let mut create_op = CreateJob::new(Some("multi-doc"));
    let attrs = client.send(&mut create_op).unwrap();
    let job_id = match *attrs.get(JOB_ATTRIBUTES_TAG, JOB_ID).unwrap().value() {
        IppValue::Integer(id) => id,
        _ => panic!("invalid value")
    };
    println!("job id: {}", job_id);

    for (i, item) in args.iter().enumerate().skip(2) {
        let last = i >= (args.len() - 1);
        println!("Sending {}, last: {}", item, last);
        let mut f = File::open(&item).unwrap();

        let mut send_op = SendDocument::new(job_id, &mut f, &env::var("USER").unwrap(), last);
        let send_attrs = client.send(&mut send_op).unwrap();
        for v in send_attrs.get_group(JOB_ATTRIBUTES_TAG).unwrap().values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
}
