extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::consts::tag::JOB_ATTRIBUTES_TAG;
use ipp::consts::attribute::JOB_ID;
use ipp::value::IppValue;

use ipp::operation::{IppOperation, CreateJob, SendDocument};

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [filename...]", args[0]);
        exit(1);
    }

    let mut create_op = CreateJob::new(&args[1], Some("multi-doc"));

    let create_attrs = create_op.execute().unwrap();

    if let &IppValue::Integer(id) = create_attrs.get(JOB_ATTRIBUTES_TAG, JOB_ID).unwrap().value() {
        println!("job id: {}", id);

        for i in 2..args.len() {
            let last = i >= (args.len() - 1);
            println!("Sending {}, last: {}", args[i], last);
            let mut f = File::open(&args[i]).unwrap();

            let mut send_op = SendDocument::new(&args[1], id, &mut f, &env::var("USER").unwrap(), last);
            let send_attrs = send_op.execute().unwrap();
            for (_, v) in send_attrs.get_group(JOB_ATTRIBUTES_TAG).unwrap() {
                println!("{}: {}", v.name(), v.value());
            }
        }
    } else {
        panic!("Invalid job id");
    }
}
