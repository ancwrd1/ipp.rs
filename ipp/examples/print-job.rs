use std::{env, error::Error, process::exit};

use futures::Future;

use ipp::client::{IppClientBuilder, IppError};
use ipp::proto::{ipp::DelimiterTag, IppAttribute, IppOperationBuilder, IppValue};

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let mut runtime = tokio::runtime::Runtime::new()?;

    let fut = tokio::fs::File::open(args[2].to_owned())
        .map_err(IppError::from)
        .and_then(move |f| {
            let mut builder = IppOperationBuilder::print_job(f)
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

            client.send(operation).and_then(|attrs| {
                for v in attrs.groups_of(DelimiterTag::JobAttributes)[0].attributes().values() {
                    println!("{}: {}", v.name(), v.value());
                }
                Ok(())
            })
        });

    Ok(runtime.block_on(fut)?)
}
