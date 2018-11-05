extern crate env_logger;
extern crate ippclient;
extern crate ippproto;

use std::env;
use std::process::exit;

use ippclient::client::IppClient;
use ippproto::operation::GetPrinterAttributes;

pub fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let client = IppClient::new(&args[1]);
    let operation = GetPrinterAttributes::with_attributes(&args[2..]);

    let attrs = client.send(operation).unwrap();

    for v in attrs.get_printer_attributes().unwrap().values() {
        println!("{}: {}", v.name(), v.value());
    }
}
