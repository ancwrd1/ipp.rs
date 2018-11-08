extern crate env_logger;
extern crate ippclient;

use std::env;
use std::process::exit;

use ippclient::{util, IppError};

fn main() {
    env_logger::init();

    let result = util::util_main(env::args_os());
    match result {
        Ok(_) => {}
        Err(e) => match e {
            IppError::ParamError(e) => e.exit(),
            _ => {
                eprintln!("ERROR: {}", e);
                exit(e.as_exit_code());
            }
        },
    }
}
