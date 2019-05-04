use std::{env, fs::File, io::BufReader, process::exit};

use ipp_client::IppClientBuilder;
use ipp_proto::{IppAttribute, IppOperationBuilder, IppValue};

pub fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let f = File::open(&args[2]).unwrap();

    let mut builder = IppOperationBuilder::print_job(Box::new(BufReader::new(f)))
        .user_name(&env::var("USER").unwrap_or_else(|_| String::new()))
        .job_title(&args[1]);

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

        builder = builder.attribute(IppAttribute::new(k, value));
    }

    let operation = builder.build();

    let client = IppClientBuilder::new(&args[1]).build();
    let attrs = client.send(operation).unwrap();

    for v in attrs.job_attributes().unwrap().values() {
        println!("{}: {}", v.name(), v.value());
    }
}
