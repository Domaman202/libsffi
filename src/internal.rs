use std::ffi::{c_char, CStr, CString};
use crate::error::Error;

pub fn try_str_to_c_string(input: &str) -> Result<CString, Error> {
    CString::new(input).map_err(|_| Error::RustStringToCString)
}

pub unsafe fn str_to_c_mut_char_unchecked(input: &str) -> *mut c_char {
    unsafe {
        CString::new(input)
            .map(|it| it.into_raw())
            .unwrap_unchecked()
    }
}

pub fn try_c_const_char_to_string(input: *const c_char) -> Option<String> {
    unsafe { CStr::from_ptr(input).to_str().ok().map(String::from) }
}

pub fn try_c_const_char_to_str(input: *const c_char) -> Option<&'static str> {
    unsafe { CStr::from_ptr(input).to_str().ok() }
}

pub fn cmp_c_const_char(first: *const c_char, second: *const c_char) -> bool {
    unsafe { CStr::from_ptr(first) == CStr::from_ptr(second) }
}

pub fn dup_c_const_char(text: *const c_char) -> *mut c_char {
    unsafe { CStr::from_ptr(text).to_owned().into_raw() }
}

pub fn free_c_mut_char(text: *mut c_char) {
    let _ = unsafe { CString::from_raw(text) };
}

pub fn starts_with(str: &str, first: char) -> bool {
    if let Some(char) = str.chars().nth(0) {
        char == first
    } else {
        false
    }
}

pub fn ends_with(str: &str, end: char) -> bool {
    if let Some(char) = str.chars().nth(str.len() - 1) {
        char == end
    } else {
        false
    }
}