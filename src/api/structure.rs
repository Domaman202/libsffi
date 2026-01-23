use crate::api::error::CError;
use crate::error::Error;
use crate::internal::try_c_const_char_to_str;
use crate::structure::StructType;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, c_uint, c_void};
use std::mem::forget;
use std::ptr::{drop_in_place, null_mut};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_type_alloc(r_struct_type: *mut *mut StructType, desc: *const c_char) -> *mut CError {
    unsafe {
        let desc = try_c_const_char_to_str(desc);
        let desc = if let Some(desc) = desc { desc } else { return Error::InvalidDescriptor(Some("Invalid structure descriptor".into())).into(); };
        match StructType::from_str(desc) {
            Ok(struct_type_) => {
                let struct_type = alloc(Layout::new::<StructType>()) as *mut StructType;
                struct_type.copy_from_nonoverlapping(&struct_type_, 1);
                forget(struct_type_);
                *r_struct_type = struct_type;
                null_mut()
            },
            Err(error) => error.into()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_type_size(struct_type: *const StructType) -> c_uint {
    unsafe { (*struct_type).size() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_malloc(struct_type: *const StructType) -> *mut c_void {
    unsafe { (*struct_type).malloc() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_calloc(struct_type: *const StructType) -> *mut c_void {
    unsafe { (*struct_type).calloc() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_set_raw(struct_type: *const StructType, structure: *mut c_void, index: c_uint, avalue: *const c_void) {
    unsafe { (*struct_type).set_raw(structure, index, avalue); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_get_raw(struct_type: *const StructType, structure: *const c_void, index: c_uint, rvalue: *mut c_void) {
    unsafe { (*struct_type).get_raw(structure, index, rvalue); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_free(structure: *mut c_void) {
    StructType::free(structure)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sffi_struct_type_free(struct_type: *mut StructType) {
    unsafe {
        drop_in_place(struct_type);
        dealloc(struct_type as *mut u8, Layout::new::<StructType>());
    }
}
