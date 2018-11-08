extern crate clap;
extern crate log;
extern crate num_traits;

extern crate ippclient;
extern crate ippparse;
extern crate ippproto;

use std::ffi::CStr;
use std::os::raw::c_char;

pub mod util;

unsafe fn convert_args(args: *const *const c_char) -> Vec<String> {
    let mut rc = Vec::new();

    let mut ptr = args;
    while !ptr.is_null() && !(*ptr).is_null() {
        let opt = CStr::from_ptr(*ptr).to_string_lossy().to_string();
        rc.push(opt);
        ptr = ptr.offset(1);
    }

    rc
}
/// Entry point to main utility function for externally linking applications
///
/// * `args` - zero-terminated char* argv[] array matching the ipputil entry point arguments, including application name
#[no_mangle]
pub unsafe extern "C" fn ipp_main(args: *const *const c_char) -> i32 {
    let args = convert_args(args);
    match util::util_main(args) {
        Ok(_) => 0,
        Err(e) => e.as_exit_code(),
    }
}
