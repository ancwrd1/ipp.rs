extern crate env_logger;
extern crate ippclient;
extern crate ippproto;

use std::env;
use std::process::exit;

use ippclient::IppClient;
use ippproto::IppOperationBuilder;

pub fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let client = IppClient::new(&args[1]);
    let operation = IppOperationBuilder::get_printer_attributes()
        .attributes(&args[2..])
        .build();

    let attrs = client.send(operation).unwrap();

    for v in attrs.printer_attributes().unwrap().values() {
        println!("{}: {}", v.name(), v.value());
    }
}
