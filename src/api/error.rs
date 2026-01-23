use crate::error::Error;
use crate::internal::try_str_to_c_string;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, c_uint, CString};
use std::ptr::{drop_in_place, null, null_mut};

pub struct CError {
    code: c_uint,
    message: *mut c_char,
}

pub const SFFI_NO_ERR: c_uint = 0;
pub const SFFI_RUST_STR_TO_C_STR_ERR: c_uint = 1;
pub const SFFI_C_STR_TO_RUST_STR_ERR: c_uint = 2;
pub const SFFI_LIB_OPEN_ERR: c_uint = 3;
pub const SFFI_LIB_SYMBOL_ERR: c_uint = 4;
pub const SFFI_LIB_CLOSE_ERR: c_uint = 5;
pub const SFFI_FFI_BAD_TYPEDEF_ERR: c_uint = 6;
pub const SFFI_FFI_BAD_ABI_ERR: c_uint = 7;
pub const SFFI_FFI_BAD_ARG_TYPEE_ERR: c_uint = 8;
pub const SFFI_INVALID_DESCRIPTOR_ERR: c_uint = 9;
pub const SFFI_INVALID_CAST_ERR: c_uint = 10;
pub const SFFI_INVALID_ARGUMENTS_ERR: c_uint = 11;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_error_code(error: *const CError) -> c_uint {
    unsafe {
        if error.is_null() { return SFFI_NO_ERR }
        (*error).code
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_error_msg(error: *const CError) -> *const c_char {
    unsafe {
        if error.is_null() { return null() }
        (*error).message
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_error_free(error: *mut CError) {
    unsafe {
        if error.is_null() { return }
        if !(*error).message.is_null() { let _ = CString::from_raw((*error).message); }
        drop_in_place(error);
        dealloc(error as *mut u8, Layout::new::<CError>());
    }
}

impl From<Error> for *mut CError {
    fn from(value: Error) -> Self {
        unsafe {
            let code = match value {
                Error::RustStringToCString => SFFI_RUST_STR_TO_C_STR_ERR,
                Error::CStringToRustString => SFFI_C_STR_TO_RUST_STR_ERR,
                Error::LibraryOpen(_) => SFFI_LIB_OPEN_ERR,
                Error::LibrarySymbol(_) => SFFI_LIB_SYMBOL_ERR,
                Error::LibraryClose(_) => SFFI_LIB_CLOSE_ERR,
                Error::FFIBadTypeDef => SFFI_FFI_BAD_TYPEDEF_ERR,
                Error::FFIBadABI => SFFI_FFI_BAD_ABI_ERR,
                Error::FFIBadArgType => SFFI_FFI_BAD_ARG_TYPEE_ERR,
                Error::InvalidDescriptor(_) => SFFI_INVALID_DESCRIPTOR_ERR,
                Error::InvalidCast(_) => SFFI_INVALID_CAST_ERR,
                Error::InvalidArguments(_) => SFFI_INVALID_ARGUMENTS_ERR,
            };
            let message = value.get_message();
            let message = if let Some(message) = message && let Ok(message) = try_str_to_c_string(message) { message.into_raw() } else { null_mut() };
            let error = alloc(Layout::new::<CError>()) as *mut CError;
            (*error).code = code;
            (*error).message = message;
            error
        }
    }
}