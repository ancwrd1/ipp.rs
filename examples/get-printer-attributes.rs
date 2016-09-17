extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;

use ipp::client::IppClient;
use ipp::request::IppRequest;
use ipp::attribute::IppAttribute;
use ipp::consts::operation::GET_PRINTER_ATTRIBUTES;
use ipp::consts::attribute::REQUESTED_ATTRIBUTES;
use ipp::consts::tag::{OPERATION_ATTRIBUTES_TAG, PRINTER_ATTRIBUTES_TAG};
use ipp::value::IppValue;

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let client = IppClient::new();
    let mut req = IppRequest::new(GET_PRINTER_ATTRIBUTES, &args[1]);

    let vals: Vec<IppValue> = args[2..].iter().map(|a| IppValue::Keyword(a.to_string())).collect();

    if vals.len() > 0 {
        req.set_attribute(OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(REQUESTED_ATTRIBUTES, IppValue::ListOf(vals)));
    }

    let attrs = client.send(&mut req).unwrap();
    for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
        println!("{}: {}", v.name(), v.value());
    }
}
