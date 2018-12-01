extern crate env_logger;
extern crate ipp_client;
extern crate ipp_proto;

use std::{env, process::exit};

use ipp_client::IppClientBuilder;
use ipp_proto::IppOperationBuilder;

pub fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let client = IppClientBuilder::new(&args[1]).build();
    let operation = IppOperationBuilder::get_printer_attributes()
        .attributes(&args[2..])
        .build();

    let attrs = client.send(operation).unwrap();

    for v in attrs.printer_attributes().unwrap().values() {
        println!("{}: {}", v.name(), v.value());
    }
}
