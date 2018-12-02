extern crate pretty_env_logger;
extern crate ipp_client;
extern crate ipp_util;

use std::{env, process::exit};

use ipp_client::IppError;
use ipp_util::util;

fn main() {
    pretty_env_logger::init_timed();

    let result = util::util_main(env::args_os());
    if let Err(e) = result {
        match e {
            IppError::ParamError(ref err) => {
                eprintln!("{}", err);
            }
            _ => {
                eprintln!("ERROR: {}", e);
            }
        }
        exit(e.as_exit_code());
    }
}
