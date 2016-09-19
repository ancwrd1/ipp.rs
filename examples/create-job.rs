extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use ipp::client::IppClient;
use ipp::request::IppRequest;
use ipp::attribute::IppAttribute;
use ipp::consts::operation::{CREATE_JOB, SEND_DOCUMENT};
use ipp::consts::tag::{OPERATION_ATTRIBUTES_TAG, JOB_ATTRIBUTES_TAG};
use ipp::consts::attribute::{JOB_NAME, REQUESTING_USER_NAME, JOB_ID, LAST_DOCUMENT};
use ipp::value::IppValue;

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }


    let client = IppClient::new();
    let mut create_req = IppRequest::new(CREATE_JOB, &args[1]);

    create_req.set_attribute(OPERATION_ATTRIBUTES_TAG,
        IppAttribute::new(JOB_NAME,
            IppValue::NameWithoutLanguage(args[2].to_string())));

    for arg in &args[3..] {
        let mut kv = arg.split("=");
        let (k, v) = (kv.next().unwrap(), kv.next().unwrap());

        if let Ok(iv) = v.parse::<i32>() {
            create_req.set_attribute(JOB_ATTRIBUTES_TAG, IppAttribute::new(k, IppValue::Integer(iv)));
        } else {
            create_req.set_attribute(JOB_ATTRIBUTES_TAG, IppAttribute::new(k, IppValue::Keyword(v.to_string())));
        }
    }

    let create_attrs = client.send(&mut create_req).unwrap();
    let id = create_attrs.get(JOB_ATTRIBUTES_TAG, JOB_ID).unwrap();
    println!("job id: {}", id.value());

    let mut f = File::open(&args[2]).unwrap();

    let mut send_req = IppRequest::new(SEND_DOCUMENT, &args[1]);

    send_req.set_attribute(OPERATION_ATTRIBUTES_TAG,
        IppAttribute::new(id.name(), id.value().clone()));

    send_req.set_attribute(OPERATION_ATTRIBUTES_TAG,
        IppAttribute::new(REQUESTING_USER_NAME,
            IppValue::NameWithoutLanguage(env::var("USER").unwrap().to_string())));

    send_req.set_attribute(OPERATION_ATTRIBUTES_TAG,
        IppAttribute::new(LAST_DOCUMENT,
            IppValue::Boolean(true)));

    send_req.set_payload(&mut f);

    let send_attrs = client.send(&mut send_req).unwrap();
    for (_, v) in send_attrs.get_group(JOB_ATTRIBUTES_TAG).unwrap() {
        println!("{}: {}", v.name(), v.value());
    }
}
