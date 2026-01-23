use crate::error::Error;
use std::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

const RTLD_LAZY: c_int = 0x0001;
// const RTLD_NOW: c_int = 0x0002;
const RTDL_LOCAL: c_int = 0x0000;
// const RTLD_GLOBAL: c_int = 0x0100;

pub unsafe fn open_library(path: *const c_char) -> Result<*mut c_void, Error> {
    unsafe {
        let result = dlopen(path, RTLD_LAZY | RTDL_LOCAL);
        if result.is_null() { return Error::lib_open_from_cstr(dlerror()) }
        Ok(result)
    }
}
pub unsafe fn get_symbol(handle: *mut c_void, name: *const c_char) -> Result<*const c_void, Error> {
    unsafe {
        let result = dlsym(handle, name);
        if result.is_null() { return Error::lib_symbol_from_cstr(dlerror()) }
        Ok(result)
    }
}

pub unsafe fn close_library(handle: *mut c_void) -> Result<(), Error> {
    unsafe {
        if dlclose(handle) == 0 {
            Ok(())
        } else {
            Error::lib_close_from_cstr(dlerror())
        }
    }
}