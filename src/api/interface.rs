use crate::interface::FuncHandle;
use libffi::raw::ffi_raw;
use std::ffi::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_func_call(func: *const FuncHandle, rvalue: *mut c_void, avalue: *mut *mut c_void) {
    unsafe { (*func)._call(rvalue, avalue); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_func_call_raw(func: *const FuncHandle, rvalue: *mut c_void, avalue: *mut ffi_raw) {
    unsafe { (*func)._call_raw(rvalue, avalue) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_func_as_raw(func: *const FuncHandle) -> *const c_void {
    unsafe { (*func).as_raw().as_raw() }
}