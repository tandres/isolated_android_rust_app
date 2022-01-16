use std::os::raw::{c_char};
use std::ffi::{CString, CStr};

#[no_mangle]
fn rust_hello(name: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(name) };
    let rname = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "unknown",
    };
    let rstr = format!("Hello, {}", rname);
    CString::new(rstr).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
