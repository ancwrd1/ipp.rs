extern crate ipp;

use ipp::{util, IppError};
use std::env;
use std::process::exit;

fn main() {
    let result = util::util_main(env::args_os());
    match result {
        Ok(_) => {}
        Err(e) => match e {
            IppError::ParamError(e) => e.exit(),
            other => {
                eprintln!("{}", other);
                exit(2);
            }
        }
    }
}
