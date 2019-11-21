use std::{env, process::exit};

use ipp_client::IppError;
use ipp_util::ipp_main;

fn main() {
    env_logger::init();

    let result = ipp_main(env::args_os());
    if let Err(e) = result {
        match e {
            IppError::ParamError(ref err) => {
                eprintln!("{}", err);
            }
            _ => {
                eprintln!("ERROR: {}", e);
            }
        }
        exit(1);
    }
}
