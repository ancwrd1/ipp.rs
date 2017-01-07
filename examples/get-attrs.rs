extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;

use ipp::consts::tag::PRINTER_ATTRIBUTES_TAG;
use ipp::operation::GetPrinterAttributes;
use ipp::client::IppClient;

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let client = IppClient::new(&args[1]);
    let mut operation = GetPrinterAttributes::with_attributes(&args[2..]);

    let attrs = client.send(&mut operation).unwrap();

    for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
        println!("{}: {}", v.name(), v.value());
    }
}
