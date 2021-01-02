use std::{env, error::Error, process::exit};

use ipp::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let client = IppClient::new(args[1].parse()?);
    let operation = IppOperationBuilder::cups().get_printers();

    let attrs = client.send(operation).await?;

    for group in attrs.groups_of(DelimiterTag::PrinterAttributes) {
        let name = group.attributes()["printer-name"].value();
        let uri = group.attributes()["device-uri"].value();
        let state = group.attributes()["printer-state"]
            .value()
            .as_enum()
            .and_then(|v| PrinterState::from_i32(*v))
            .ok_or(IppError::InvalidAttributeType)?;

        println!("{}: {} {:?}", name, uri, state);
    }

    Ok(())
}
