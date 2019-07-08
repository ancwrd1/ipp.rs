use std::{env, error::Error, process::exit};

use num_traits::cast::FromPrimitive;

use ipp_client::{IppClientBuilder, IppError};
use ipp_proto::{
    ipp::{DelimiterTag, PrinterState},
    operation::cups::CupsGetPrinters,
    IppValue,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let mut runtime = tokio::runtime::Runtime::new()?;
    let client = IppClientBuilder::new(&args[1]).build();
    let operation = CupsGetPrinters::new();

    let attrs = runtime.block_on(client.send(operation))?;

    for group in attrs.groups_of(DelimiterTag::PrinterAttributes) {
        let name = group.attributes()["printer-name"].clone();
        let uri = group.attributes()["device-uri"].clone();
        let state = group.attributes()["printer-state"].value();
        let state = match state {
            IppValue::Enum(e) => PrinterState::from_i32(*e).unwrap(),
            _ => return Err(IppError::ParamError("Invalid state encoding!".to_owned()).into()),
        };
        println!("{}: {} {:?}", name.value(), uri.value(), state);
    }

    Ok(())
}