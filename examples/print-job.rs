extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::client::IppClient;
use ipp::request::IppRequest;
use ipp::attribute::IppAttribute;
use ipp::consts::operation::PRINT_JOB;
use ipp::consts::tag::{OPERATION_ATTRIBUTES_TAG, JOB_ATTRIBUTES_TAG};
use ipp::consts::attribute::{JOB_NAME, REQUESTING_USER_NAME};
use ipp::value::IppValue;

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let mut f = File::open(&args[2]).unwrap();

    let client = IppClient::new();
    let mut req = IppRequest::new(PRINT_JOB, &args[1]);

    for arg in &args[3..] {
        let mut kv = arg.split("=");
        let (k, v) = (kv.next().unwrap(), kv.next().unwrap());

        if let Ok(iv) = v.parse::<i32>() {
            req.set_attribute(JOB_ATTRIBUTES_TAG, IppAttribute::new(k, IppValue::Integer(iv)));
        } else {
            req.set_attribute(JOB_ATTRIBUTES_TAG, IppAttribute::new(k, IppValue::Keyword(v.to_string())));
        }
    }

    req.set_attribute(OPERATION_ATTRIBUTES_TAG,
        IppAttribute::new(JOB_NAME,
            IppValue::NameWithoutLanguage(args[2].to_string())));

    req.set_attribute(OPERATION_ATTRIBUTES_TAG,
        IppAttribute::new(REQUESTING_USER_NAME,
            IppValue::NameWithoutLanguage(env::var("USER").unwrap().to_string())));

    req.set_payload(&mut f);

    let attrs = client.send(&mut req).unwrap();
    for (_, v) in attrs.get_group(JOB_ATTRIBUTES_TAG).unwrap() {
        println!("{}: {}", v.name(), v.value());
    }
}
