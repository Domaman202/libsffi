use crate::interface::FuncHandle;
use crate::internal::{cmp_c_const_char, dup_c_const_char, free_c_mut_char, try_str_to_c_string};
use crate::platform::platform;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, c_void};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::{null, null_mut};
use crate::error::Error;

#[derive(Debug)]
pub struct LibHandle {
    pub handle: *mut c_void,
    pub functions: *mut LibHandleFuncList
}

#[derive(Debug)]
pub struct LibHandleFuncList {
    next: *mut LibHandleFuncList,
    name: *mut c_char,
    func: ManuallyDrop<FuncHandle>
}

impl LibHandle {
    pub(crate) fn new(handle: *mut c_void) -> Self {
        Self {
            handle,
            functions: null_mut()
        }
    }

    pub fn open(name: &str) -> Result<LibHandle, Error> {
        unsafe {
            let name = try_str_to_c_string(name)?;
            Self::_open(name.as_ptr())
        }
    }

    pub(crate) unsafe fn _open(name: *const c_char) -> Result<LibHandle, Error> {
        unsafe {
            let handle = platform::open_library(name)?;
            Ok(LibHandle::new(handle))
        }
    }

    pub fn symbol(&self, name: &str) -> Result<*const c_void, Error> {
        unsafe {
            let name = try_str_to_c_string(name)?;
            self._symbol(name.as_ptr())
        }
    }

    pub(crate) unsafe fn _symbol(&self, name: *const c_char) -> Result<*const c_void, Error> {
        unsafe { crate::platform::unix::get_symbol(self.handle, name) }
    }

    pub fn func(&mut self, name: &str, desc: &str) -> Result<&FuncHandle, Error> {
        unsafe {
            let name = try_str_to_c_string(name)?;
            let func = self._func(name.as_ptr(), desc)?;
            Ok(&*func)
        }
    }

    pub(crate) unsafe fn _func(&mut self, name: *const c_char, desc: &str) -> Result<*const FuncHandle, Error> {
        unsafe {
            let find = self.find_function(name);
            if !find.is_null() { return Ok(find) }
            let symbol = self._symbol(name)?;
            let func = FuncHandle::new(symbol, desc)?;
            let node = alloc(Layout::new::<LibHandleFuncList>()) as *mut LibHandleFuncList;
            (*node).name = dup_c_const_char(name);
            (*node).func = ManuallyDrop::new(func);
            (*node).next = self.functions;
            self.functions = node;
            Ok((*node).func.deref())
        }
    }

    fn find_function(&self, name: *const c_char) -> *const FuncHandle {
        unsafe {
            let mut last_node = self.functions;
            loop {
                if last_node.is_null() { return null(); }
                if cmp_c_const_char((*last_node).name, name) { return (*last_node).func.deref() }
                last_node = (*last_node).next;
            }
        }
    }

    pub fn as_raw(&self) -> *mut c_void {
        self.handle
    }
}

impl Drop for LibHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = crate::platform::unix::close_library(self.handle);
            let mut last_node = self.functions;
            loop {
                if last_node.is_null() { break }
                let next = (*last_node).next;
                free_c_mut_char((*last_node).name);
                ManuallyDrop::drop(&mut (*last_node).func);
                dealloc(last_node as *mut _, Layout::new::<LibHandleFuncList>());
                last_node = next;
            }
        }
    }
}