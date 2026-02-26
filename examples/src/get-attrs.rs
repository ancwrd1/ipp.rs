use std::{env, error::Error, process::exit};

use ipp::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let uri: Uri = args[1].parse()?;
    let client = IppClient::new(uri.clone());
    let operation = IppOperationBuilder::get_printer_attributes(uri)
        .attributes(&args[2..])
        .build()?;

    let response = client.send(operation)?;
    println!("IPP status code: {}", response.header().status_code());

    let attrs = response
        .attributes()
        .groups_of(DelimiterTag::PrinterAttributes)
        .flat_map(|g| g.attributes().values());

    for attr in attrs {
        println!("{}: {}", attr.name(), attr.value());
    }

    Ok(())
}
