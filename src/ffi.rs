use std::ffi::CStr;
use std::os::raw::c_char;
use util;

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

#[no_mangle]
pub unsafe extern fn ipp_main(args: *const *const c_char) -> i32 {
    let args = convert_args(args);
    match util::util_main(args) {
        Ok(_) => 0,
        Err(e) => e.as_exit_code()
    }
}
