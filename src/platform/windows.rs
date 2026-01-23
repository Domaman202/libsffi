use std::ffi::{c_char, c_void, CStr};
use std::ptr::{null, null_mut};
use crate::error::Error;

unsafe extern "system" {
    fn LoadLibraryA(name: *const c_char) -> *mut c_void;
    fn GetProcAddress(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(handle: *mut c_void) -> i32;
    fn GetLastError() -> u32;
    fn FormatMessageA(flags: u32, src: *const c_void, msg_id: u32, lang_id: u32, buff: *mut i8, size: u32, args: *const c_void) -> u32;
    fn LocalFree(handle: *mut c_void);
}

const FORMAT_MESSAGE_ALLOCATE_BUFFER: u32 = 0x00000100;
const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x00001000;
const FORMAT_MESSAGE_IGNORE_INSERTS: u32 = 0x00000200;

pub unsafe fn open_library(path: *const c_char) -> Result<*mut c_void, Error> {
    unsafe {
        let result = LoadLibraryA(path);
        if result.is_null() { return Error::lib_open_from_string(get_error_message(GetLastError())) }
        Ok(result)
    }
}
pub unsafe fn get_symbol(handle: *mut c_void, name: *const c_char) -> Result<*mut c_void, Error> {
    unsafe {
        let result = GetProcAddress(handle, name);
        if result.is_null() { return Error::lib_symbol_from_string(get_error_message(GetLastError())) }
        Ok(result)
    }
}

pub unsafe fn close_library(handle: *mut c_void) -> Result<(), Error> {
    unsafe {
        if FreeLibrary(handle) == 0 {
            Error::lib_close_from_string(get_error_message(GetLastError()))
        } else {
            Ok(())
        }
    }
}

fn get_error_message(error_code: u32) -> String {
    unsafe {
        let mut buffer: *mut i8 = null_mut();

        let chars = FormatMessageA(
            FORMAT_MESSAGE_ALLOCATE_BUFFER |
                FORMAT_MESSAGE_FROM_SYSTEM |
                FORMAT_MESSAGE_IGNORE_INSERTS,
            null(),
            error_code,
            0,
            &mut buffer as *mut *mut i8 as *mut i8,
            0,
            null(),
        );

        if chars == 0 {
            return format!("Unknown error ({})", error_code);
        }

        let message = CStr::from_ptr(buffer)
            .to_string_lossy()
            .trim()
            .to_string();

        LocalFree(buffer as *mut c_void);

        message
    }
}
