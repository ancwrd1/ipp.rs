use std::{env, error::Error, fs, process::exit};

use ipp::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let uri: Uri = args[1].parse()?;
    let payload = IppPayload::new(fs::File::open(&args[2])?);

    let mut builder = IppOperationBuilder::print_job(uri.clone(), payload)
        .user_name(&env::var("USER").unwrap_or_else(|_| "noname".to_owned()))
        .job_title(&args[1]);

    for arg in &args[3..] {
        let mut kv = arg.split('=');
        let (k, v) = (kv.next().unwrap(), kv.next().unwrap());

        builder = builder.attribute(IppAttribute::new(k, v.parse()?));
    }

    let operation = builder.build();

    let client = IppClient::new(uri);

    let response = client.send(operation).await?;

    println!("IPP status code: {}", response.header().status_code());

    for v in response
        .attributes()
        .groups_of(DelimiterTag::JobAttributes)
        .map(|g| g.attributes().values())
        .flatten()
    {
        println!("{}: {}", v.name(), v.value());
    }

    Ok(())
}
