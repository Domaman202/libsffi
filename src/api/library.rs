use crate::api::error::CError;
use crate::error::Error;
use crate::interface::FuncHandle;
use crate::internal::try_c_const_char_to_str;
use crate::library::LibHandle;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, c_void};
use std::mem::forget;
use std::ptr::{drop_in_place, null_mut};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_lib_open(r_handle: *mut *mut LibHandle, name: *const c_char) -> *mut CError {
    unsafe {
        match LibHandle::_open(name) {
            Ok(handle_) => {
                let handle = alloc(Layout::new::<LibHandle>()) as *mut LibHandle;
                handle.copy_from_nonoverlapping(&handle_, 1);
                forget(handle_);
                *r_handle = handle;
                null_mut()
            },
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_lib_symbol(r_symbol: *mut *const c_void, handle: *const LibHandle, name: *const c_char) -> *mut CError {
    unsafe {
        match (*handle)._symbol(name) {
            Ok(symbol) => { *r_symbol = symbol; null_mut() },
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_lib_func(r_func: *mut *const FuncHandle, handle: *mut LibHandle, name: *const c_char, desc: *const c_char) -> *mut CError {
    unsafe {
        let desc = try_c_const_char_to_str(desc);
        let desc = if let Some(desc) = desc { desc } else { return Error::InvalidDescriptor(Some("Invalid platform function descriptor".into())).into() };
        match (*handle)._func(name, desc) {
            Ok(func) => { *r_func = func; null_mut() },
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_lib_as_raw(handle: *const LibHandle) -> *mut c_void {
    unsafe { (*handle).as_raw() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_lib_close(handle: *mut LibHandle) {
    unsafe {
        drop_in_place(handle);
        dealloc(handle as *mut _, Layout::new::<LibHandle>());
    }
}