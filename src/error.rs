use crate::internal::try_c_const_char_to_string;
use std::ffi::c_char;

#[derive(Debug)]
pub enum Error {
    RustStringToCString,
    CStringToRustString,
    LibraryOpen(Option<String>),
    LibrarySymbol(Option<String>),
    LibraryClose(Option<String>),
    FFIBadTypeDef,
    FFIBadABI,
    FFIBadArgType,
    InvalidDescriptor(Option<String>),
    InvalidCast(Option<String>),
    InvalidArguments(Option<String>),
}


impl Error {
    pub fn get_message(&self) -> Option<&str> {
        match self {
            Error::RustStringToCString |
            Error::CStringToRustString |
            Error::FFIBadTypeDef |
            Error::FFIBadABI |
            Error::FFIBadArgType
            => None,
            Error::LibraryOpen(str) |
            Error::LibrarySymbol(str) |
            Error::LibraryClose(str) |
            Error::InvalidDescriptor(str) |
            Error::InvalidCast(str) |
            Error::InvalidArguments(str)
            => if let Some(str) = str { Some(&str) } else { None },
        }
    }

    pub(crate) fn lib_open_from_cstr<T>(str: *const c_char) -> Result<T, Error> {
        Err(Error::LibraryOpen(try_c_const_char_to_string(str)))
    }

    pub(crate) fn lib_symbol_from_cstr<T>(str: *const c_char) -> Result<T, Error> {
        Err(Error::LibrarySymbol(try_c_const_char_to_string(str)))
    }

    pub(crate) fn lib_close_from_cstr<T>(str: *const c_char) -> Result<T, Error> {
        Err(Error::LibraryClose(try_c_const_char_to_string(str)))
    }

    pub(crate) fn invalid_desc_from_str<T>(str: &str) -> Result<T, Error> {
        Err(Error::InvalidDescriptor(Some(str.into())))
    }

    pub(crate) fn invalid_desc_from_string<T>(str: String) -> Result<T, Error> {
        Err(Error::InvalidDescriptor(Some(str)))
    }

    pub(crate) fn invalid_cast_from_string<T>(str: String) -> Result<T, Error> {
        Err(Error::InvalidCast(Some(str)))
    }

    pub(crate) fn invalid_args_from_string<T>(str: String) -> Result<T, Error> {
        Err(Error::InvalidArguments(Some(str)))
    }
}