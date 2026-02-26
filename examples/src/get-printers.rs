use std::{env, error::Error, process::exit};

use ipp::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let client = IppClient::builder(args[1].parse()?).build();
    let cups = IppOperationBuilder::cups();
    let operation = cups.get_printers();

    let response = client.send(operation)?;
    println!("IPP status code: {}", response.header().status_code());

    for group in response.attributes().groups_of(DelimiterTag::PrinterAttributes) {
        let name = group.attributes()["printer-name"].value();
        let uri = group.attributes()["device-uri"].value();
        let state = group.attributes()["printer-state"]
            .value()
            .as_enum()
            .and_then(|v| PrinterState::from_i32(*v))
            .ok_or(IppError::InvalidAttributeType)?;

        println!("{name}: {uri} {state:?}");
    }

    Ok(())
}
