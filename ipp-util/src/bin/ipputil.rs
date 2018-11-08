extern crate env_logger;
extern crate ippclient;
extern crate ipputil;

use std::env;
use std::process::exit;

use ippclient::IppError;
use ipputil::util;

fn main() {
    env_logger::init();

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
