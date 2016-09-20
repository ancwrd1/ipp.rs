extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;

use ipp::consts::tag::PRINTER_ATTRIBUTES_TAG;
use ipp::operation::{IppOperation, GetPrinterAttributes};

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let mut operation = GetPrinterAttributes::new(&args[1]);
    operation.set_attributes(&args[2..]);

    let attrs = operation.execute().unwrap();

    for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
        println!("{}: {}", v.name(), v.value());
    }
}
