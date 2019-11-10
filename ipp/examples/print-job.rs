use std::{env, error::Error, process::exit};

use ipp::{
    client::IppClientBuilder,
    proto::{ipp::DelimiterTag, IppAttribute, IppOperationBuilder, IppValue},
};
use ipp_client::TokioReadAdapter;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let reader = TokioReadAdapter::new(tokio::fs::File::open(&args[2]).await?);

    let mut builder = IppOperationBuilder::print_job(reader)
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

    let attrs = client.send(operation).await?;
    for v in attrs.groups_of(DelimiterTag::JobAttributes)[0].attributes().values() {
        println!("{}: {}", v.name(), v.value());
    }

    Ok(())
}
