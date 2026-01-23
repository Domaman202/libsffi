use crate::adapter::Adapter;
use crate::api::error::CError;
use crate::error::Error;
use crate::interface::FuncHandle;
use crate::internal::try_c_const_char_to_str;
use crate::structure::StructType;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, c_uint, c_void};
use std::mem::forget;
use std::ptr::{drop_in_place, null_mut};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_adapter_parse(r_adapter: *mut *mut Adapter, desc: *const c_char) -> *mut CError {
    unsafe {
        let desc = try_c_const_char_to_str(desc);
        let desc = if let Some(desc) = desc { desc } else { return Error::InvalidDescriptor(Some("Invalid adapter descriptor".into())).into(); };
        match Adapter::from_str(desc) {
            Ok(adapter_) => {
                let adapter = alloc(Layout::new::<Adapter>()) as *mut Adapter;
                adapter.copy_from_nonoverlapping(&adapter_, 1);
                forget(adapter_);
                *r_adapter = adapter;
                null_mut()
            }
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_adapter_call(adapter: *const Adapter, func: *const FuncHandle, rvalue: *mut c_void, argc: c_uint, argv: *mut *mut c_void) -> *mut CError {
    unsafe {
        match (*adapter)._call(func, rvalue, argc, argv) {
            Ok(_) => null_mut(),
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_adapter_set(adapter: *const Adapter, struct_type: *const StructType, structure: *mut c_void, index: c_uint, avalue: *const c_void) -> *mut CError {
    unsafe {
        match (*adapter)._set(struct_type, structure, index, avalue) {
            Ok(_) => null_mut(),
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_adapter_get(adapter: *const Adapter, struct_type: *const StructType, structure: *const c_void, index: c_uint, rvalue: *mut c_void) -> *mut CError {
    unsafe {
        match (*adapter)._get(struct_type, structure, index, rvalue) {
            Ok(_) => null_mut(),
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_adapter_free(adapter: *mut Adapter) {
    unsafe {
        drop_in_place(adapter);
        dealloc(adapter as *mut u8, Layout::new::<Adapter>());
    }
}