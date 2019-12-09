use std::{env, error::Error, process::exit};

use ipp::prelude::*;

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        exit(1);
    }

    let client = IppClientBuilder::new(&args[1]).build();
    let operation = IppOperationBuilder::get_printer_attributes()
        .attributes(&args[2..])
        .build();

    let attrs = futures::executor::block_on(client.send(operation))?;

    for v in attrs.groups_of(DelimiterTag::PrinterAttributes)[0]
        .attributes()
        .values()
    {
        println!("{}: {}", v.name(), v.value());
    }

    Ok(())
}
