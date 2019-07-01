use std::{env, process::exit};

use ipp_client::IppClientBuilder;
use ipp_proto::operation::cups::CupsDeletePrinter;

pub fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let client = IppClientBuilder::new(&args[1]).build();
    let operation = CupsDeletePrinter::new();

    runtime.block_on(client.send(operation)).unwrap();
}
