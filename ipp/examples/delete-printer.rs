use std::{env, error::Error, process::exit};

use ipp::client::IppClientBuilder;
use ipp::proto::operation::cups::CupsDeletePrinter;

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let mut runtime = tokio::runtime::Runtime::new()?;
    let client = IppClientBuilder::new(&args[1]).build();
    let operation = CupsDeletePrinter::new();

    runtime.block_on(client.send(operation))?;

    Ok(())
}
