extern crate env_logger;
extern crate ipp;

use std::env;
use std::fs::File;
use std::process::exit;

use ipp::{IppAttribute, IppClient, IppValue, PrintJob};

pub fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let client = IppClient::new(&args[1]);
    let f = File::open(&args[2]).unwrap();
    let mut operation = PrintJob::new(Box::new(f), &env::var("USER").unwrap(), Some(&args[1]));

    for arg in &args[3..] {
        let mut kv = arg.split('=');
        let (k, v) = (kv.next().unwrap(), kv.next().unwrap());

        let value = if let Ok(iv) = v.parse::<i32>() {
            IppValue::Integer(iv)
        } else if v == "true" || v == "false" {
            IppValue::Boolean(v == "true")
        } else {
            IppValue::Keyword(v.to_string())
        };

        operation.add_attribute(IppAttribute::new(k, value));
    }

    let attrs = client.send(operation).unwrap();

    for v in attrs.get_job_attributes().unwrap().values() {
        println!("{}: {}", v.name(), v.value());
    }
}
