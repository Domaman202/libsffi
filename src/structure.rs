use crate::interface::FuncType;
use std::ffi::{c_uint, c_void};
use std::{fmt, ptr};
use std::fmt::{Display, Formatter};
use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructType(Box<[(FuncType, c_uint)]>, c_uint);

impl StructType {
    pub fn new(fields: Box<[FuncType]>) -> Self {
        let mut offset = 0 as c_uint;
        let mut fields_with_offset = Box::new(Vec::with_capacity(fields.len()));
        for field in fields {
            let size = field.size();
            fields_with_offset.push((field, offset));
            offset += size;
        }
        Self(fields_with_offset.into_boxed_slice(), offset)
    }

    pub fn from_str(str: &str) -> Result<Self, Error> {
        Ok(Self::_from_str(str)?.1)
    }

    pub(crate) fn _from_str(str: &str) -> Result<(&str, Self), Error> {
        let mut str = &str[1..];
        let mut fields = vec![];
        while !str.starts_with("]") {
            if str.starts_with("[") {
                let (str_, value) = Self::_from_str(str)?;
                str = str_;
                fields.push(FuncType::Struct(value));
            } else {
                let next = if let Some(next) = str.find(",") { Some(next) } else { str.find("]") };
                let next = if let Some(next) = next { next } else { return Error::invalid_desc_from_str("Struct without end") };
                let (_, value) = FuncType::__from_str(&str[..next])?;
                fields.push(value);
                str = &str[next..];
                if str.is_empty() { break; }
            }
            if str.starts_with(",") {
                str =&str[1..]
            };
        }
        Ok((&str[1..], StructType::new(fields.into_boxed_slice())))
    }

    pub fn fields(&self) -> &[(FuncType, c_uint)] {
        &self.0
    }

    pub fn size(&self) -> c_uint {
        self.1
    }

    pub fn malloc(&self) -> *mut c_void {
        unsafe extern "C" { fn malloc(size: c_uint) -> *mut c_void; }
        unsafe { malloc(self.size()) }
    }

    pub fn calloc(&self) -> *mut c_void {
        unsafe extern "C" { fn calloc(size: c_uint, count: c_uint) -> *mut c_void; }
        unsafe { calloc(self.size(), 1) }
    }

    pub unsafe fn set_raw(&self, structure: *mut c_void, index: c_uint, avalue: *const c_void) {
        if let Some((field, offset)) = self.0.get(index as usize) {
            unsafe {
                ptr::copy_nonoverlapping::<u8>(
                    avalue as *const u8,
                    structure.byte_offset(*offset as isize) as *mut u8,
                    field.size() as usize
                )
            }
        }
    }

    pub unsafe fn get_raw(&self, structure: *const c_void, index: c_uint, rvalue: *mut c_void) {
        if let Some((field, offset)) = self.0.get(index as usize) {
            unsafe {
                ptr::copy_nonoverlapping::<u8>(
                    structure.byte_offset(*offset as isize) as *const u8,
                    rvalue as *mut u8,
                    field.size() as usize
                )
            }
        }
    }

    pub fn free(ptr: *mut c_void) {
        unsafe extern "C" { fn free(ptr: *mut c_void); }
        unsafe { free(ptr); }
    }
}

impl Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, (field_type, offset)) in self.0.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}@{:#x}", field_type, offset)?;
        }
        write!(f, "] (size: {})", self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(FuncType::Struct(StructType::from_str("[]").unwrap()), FuncType::structure(&[]));
        assert_eq!(FuncType::Struct(StructType::from_str("[i32,f32]").unwrap()), FuncType::structure(&[FuncType::S32, FuncType::F32]));
    }

    #[test]
    fn test_inner() {
        assert_eq!(FuncType::Struct(StructType::from_str("[[]]").unwrap()), FuncType::structure(&[FuncType::structure(&[])]));
        assert_eq!(FuncType::Struct(StructType::from_str("[i32,[f32]]").unwrap()), FuncType::structure(&[FuncType::S32, FuncType::structure(&[FuncType::F32])]));
        assert_eq!(FuncType::Struct(StructType::from_str("[[],[]]").unwrap()), FuncType::structure(&[FuncType::structure(&[]), FuncType::structure(&[])]));
    }
}