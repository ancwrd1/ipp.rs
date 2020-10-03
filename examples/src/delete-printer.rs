use std::{env, error::Error, process::exit};

use ipp::prelude::*;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let uri: Uri = args[1].parse()?;
    let client = IppClient::new(uri.clone());
    let operation = IppOperationBuilder::cups().delete_printer(uri);

    futures::executor::block_on(client.send(operation))?;

    Ok(())
}
