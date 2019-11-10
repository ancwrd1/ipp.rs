use std::{env, error::Error, process::exit};

use ipp::{client::IppClientBuilder, proto::operation::cups::CupsDeletePrinter};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let client = IppClientBuilder::new(&args[1]).build();
    let operation = CupsDeletePrinter::new();

    client.send(operation).await?;

    Ok(())
}
