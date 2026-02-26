use std::{env, error::Error, fs, process::exit};

use ipp::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let uri: Uri = args[1].parse()?;
    let payload = IppPayload::new(fs::File::open(&args[2])?);

    let mut builder = IppOperationBuilder::print_job(uri.clone(), payload)
        .user_name(env::var("USER").unwrap_or_else(|_| "noname".to_owned()))
        .job_title(&args[1]);

    for arg in &args[3..] {
        if let Some((k, v)) = arg.split_once('=') {
            builder = builder.attribute(IppAttribute::new(k.try_into()?, v.parse()?));
        }
    }

    let operation = builder.build()?;
    let client = IppClient::new(uri);
    let response = client.send(operation)?;

    println!("IPP status code: {}", response.header().status_code());

    let attrs = response
        .attributes()
        .groups_of(DelimiterTag::JobAttributes)
        .flat_map(|g| g.attributes().values());

    for attr in attrs {
        println!("{}: {}", attr.name(), attr.value());
    }

    Ok(())
}
