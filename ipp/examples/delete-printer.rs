use std::{env, error::Error, process::exit};

use ipp::client::IppClientBuilder;
use ipp_proto::IppOperationBuilder;

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let client = IppClientBuilder::new(&args[1]).build();
    let operation = IppOperationBuilder::cups().delete_printer();

    futures::executor::block_on(client.send(operation))?;

    Ok(())
}
