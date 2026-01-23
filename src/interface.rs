use crate::library::LibSymbol;
use libffi::high::ffi_abi_FFI_DEFAULT_ABI;
use libffi::low::ffi_cif;
use libffi::raw::{ffi_call, ffi_prep_cif, ffi_raw, ffi_raw_call, ffi_status_FFI_BAD_ABI, ffi_status_FFI_BAD_ARGTYPE, ffi_status_FFI_BAD_TYPEDEF, ffi_status_FFI_OK, ffi_type, ffi_type_double, ffi_type_float, ffi_type_longdouble, ffi_type_pointer, ffi_type_sint16, ffi_type_sint32, ffi_type_sint64, ffi_type_sint8, ffi_type_uint16, ffi_type_uint32, ffi_type_uint64, ffi_type_uint8, ffi_type_void, FFI_TYPE_STRUCT};
use std::cmp::min;
use std::ffi::{c_double, c_float, c_int, c_uint, c_void};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::mem::transmute;
use std::ptr::null_mut;
use crate::error::Error;
use crate::internal::starts_with;
use crate::structure::StructType;

#[derive(Debug)]
pub struct FuncHandle {
    desc: FuncDesc,
    symbol: LibSymbol
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncDesc {
    argument_types: Box<[FuncType]>,
    return_type: FuncType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuncType {
    Auto, Void,
    Int, Float, Double, LongDouble, ISize, USize,
    S8, S16, S32, S64, U8, U16, U32, U64, F32, F64, F128,
    Pointer,
    RefStringPtr, BorrowStringPtr,
    RefArrayPtr, BorrowArrayPtr,
    Struct(StructType)
}

struct FuncDescHelper {
    _boxed: Vec<(Box<ffi_type>, Box<Vec<*mut ffi_type>>)>,
    return_type: *mut ffi_type,
    arguments_types: Box<[*mut ffi_type]>,
}

impl FuncHandle {
    pub(crate) fn new(symbol: LibSymbol, desc: &str) -> Result<Self, Error> {
        let desc = FuncDesc::from_str(desc)?;

        // Verify
        let mut helper = FuncDescHelper::new(&desc.return_type, &desc.argument_types)?;
        let mut cif = ffi_cif::default();
        #[allow(nonstandard_style)]
        match unsafe {
            ffi_prep_cif(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                helper.arguments_types.len() as c_uint,
                helper.return_type,
                helper.arguments_types.as_mut_ptr(),
            )
        } {
            ffi_status_FFI_OK => {},
            ffi_status_FFI_BAD_TYPEDEF => return Err(Error::FFIBadTypeDef),
            ffi_status_FFI_BAD_ABI => return Err(Error::FFIBadABI),
            ffi_status_FFI_BAD_ARGTYPE => return Err(Error::FFIBadArgType),
            _ => unreachable!()
        };

        Ok(Self { desc, symbol })
    }

    pub fn desc(&self) -> &FuncDesc {
        &self.desc
    }

    pub unsafe fn call(&self, rvalue: *mut c_void, avalue: &mut [*mut c_void]) {
        unsafe { self._call(rvalue, avalue.as_mut_ptr()) }
    }

    pub(crate) unsafe fn _call(&self, rvalue: *mut c_void, avalue: *mut *mut c_void) {
        unsafe {
            let mut cif = ffi_cif::default();
            let mut helper = FuncDescHelper::new(&self.desc.return_type, &self.desc.argument_types).unwrap_unchecked();

            #[allow(nonstandard_style)]
            ffi_prep_cif(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                helper.arguments_types.len() as c_uint,
                helper.return_type,
                helper.arguments_types.as_mut_ptr(),
            );

            ffi_call(
                &mut cif,
                transmute(self.symbol.as_raw()),
                rvalue,
                avalue
            );
        }
    }

    pub unsafe fn call_raw(&self, result: *mut c_void, arguments: &mut [ffi_raw]) {
        unsafe { self._call_raw(result, arguments.as_mut_ptr()) }
    }

    pub(crate) unsafe fn _call_raw(&self, result: *mut c_void, arguments: *mut ffi_raw) {
        unsafe {
            let mut cif = ffi_cif::default();
            let mut helper = FuncDescHelper::new(&self.desc.return_type, &self.desc.argument_types).unwrap_unchecked();

            #[allow(nonstandard_style)]
            ffi_prep_cif(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                helper.arguments_types.len() as c_uint,
                helper.return_type,
                helper.arguments_types.as_mut_ptr(),
            );

            ffi_raw_call(
                &mut cif,
                transmute(self.symbol.as_raw()),
                result,
                arguments
            );
        }
    }
}

impl FuncDesc {
    pub fn new(argument_types: Box<[FuncType]>, return_type: FuncType) -> Self {
        Self {
            argument_types,
            return_type,
        }
    }

    pub fn from_str(str: &str) -> Result<FuncDesc, Error> {
        let str = str.trim();
        if !starts_with(str, '(') { return Error::invalid_desc_from_str("Invalid descriptor start") }
        let str = &str[1..];
        let end_index = str.find(')');
        let end_index = if let Some(idx) = end_index { idx } else { return Error::invalid_desc_from_str("Invalid descriptor end") };
        let return_type = FuncType::from_str(&str[end_index + 1..str.len()])?;
        let argument_types = FuncType::from_str_list(&str[..end_index])?.into_boxed_slice();
        Ok(
            FuncDesc {
                argument_types,
                return_type
            }
        )
    }

    pub fn argument_types(&self) -> &'_[FuncType] {
        &self.argument_types
    }

    pub fn return_type(&self) -> &FuncType {
        &self.return_type
    }
}

impl FuncType {
    pub fn structure(fields: &[FuncType]) -> Self {
        Self::Struct(StructType::new(Vec::from(fields).into_boxed_slice()))
    }

    pub fn from_str_list(str: &str) -> Result<Vec<Self>, Error> {
        let mut str = str;
        let mut list = vec![];
        while !str.is_empty() {
            let (str_, value) = Self::__from_str(str)?;
            str = str_;
            list.push(value);
        }
        Ok(list)
    }

    pub fn from_str(str: &str) -> Result<Self, Error> {
        Ok(Self::__from_str(str)?.1)
    }

    pub(crate) fn __from_str(str: &str) -> Result<(&str, Self), Error> {
        if str.starts_with("[") {
            let (str, structure) = StructType::_from_str(str)?;
            Ok((str, FuncType::Struct(structure)))
        } else {
            let next = if let Some(next) = str.find(",") { next } else { str.len() };
            let value = Self::_from_str(&str[..next]);
            let value = if let Some(value) = value { value } else { return Error::invalid_desc_from_string(format!("Unknown type: {}", &str[..next])) };
            Ok((&str[min(next + 1, str.len())..], value))
        }
    }

    pub(crate) fn _from_str(str: &str) -> Option<Self> {
        Some(
            match str {
                "auto"|"?"  => FuncType::Auto,
                "void"      => FuncType::Void,

                "int"       => FuncType::Int,
                "float"     => FuncType::Float,
                "double"    => FuncType::Double,
                "longdouble"=> FuncType::LongDouble,
                "isize"     => FuncType::ISize,
                "usize"     => FuncType::USize,

                "i8"        => FuncType::S8,
                "i16"       => FuncType::S16,
                "i32"       => FuncType::S32,
                "i64"       => FuncType::S64,

                "u8"        => FuncType::U8,
                "u16"       => FuncType::U16,
                "u32"       => FuncType::U32,
                "u64"       => FuncType::U64,

                "f32"       => FuncType::F32,
                "f64"       => FuncType::F64,
                "f128"      => FuncType::F128,

                "*"         => FuncType::Pointer,
                "&str"      => FuncType::RefStringPtr,
                "*str"      => FuncType::BorrowStringPtr,
                "&[]"       => FuncType::RefArrayPtr,
                "*[]"       => FuncType::BorrowArrayPtr,

                _ => return None
            }
        )
    }

    pub fn is_auto(&self) -> bool {
        match self {
            FuncType::Auto => true,
            _ => false
        }
    }

    pub fn is_ptr(&self) -> bool {
        match self {
            FuncType::ISize |
            FuncType::USize |
            FuncType::Pointer |
            FuncType::RefStringPtr |
            FuncType::BorrowStringPtr |
            FuncType::RefArrayPtr |
            FuncType::BorrowArrayPtr => true,
            _ => false
        }
    }

    pub fn is_ref_str(&self) -> bool {
        match self {
            FuncType::RefStringPtr => true,
            _ => false
        }
    }

    pub fn is_borrow_str(&self) -> bool {
        match self {
            FuncType::BorrowStringPtr => true,
            _ => false
        }
    }

    pub fn size(&self) -> c_uint {
        match self {
            FuncType::Auto              => 0                    as c_uint,
            FuncType::Void              => 0                    as c_uint,
            FuncType::Int               => size_of::<c_int>()   as c_uint,
            FuncType::Float             => size_of::<c_float>() as c_uint,
            FuncType::Double            => size_of::<c_double>()as c_uint,
            FuncType::LongDouble        => size_of::<i128>()    as c_uint,
            FuncType::ISize             => size_of::<isize>()   as c_uint,
            FuncType::USize             => size_of::<usize>()   as c_uint,
            FuncType::S8                => size_of::<i8>()      as c_uint,
            FuncType::S16               => size_of::<i16>()     as c_uint,
            FuncType::S32               => size_of::<i32>()     as c_uint,
            FuncType::S64               => size_of::<i64>()     as c_uint,
            FuncType::U8                => size_of::<u8>()      as c_uint,
            FuncType::U16               => size_of::<u16>()     as c_uint,
            FuncType::U32               => size_of::<u32>()     as c_uint,
            FuncType::U64               => size_of::<u64>()     as c_uint,
            FuncType::F32               => size_of::<f32>()     as c_uint,
            FuncType::F64               => size_of::<f64>()     as c_uint,
            FuncType::F128              => size_of::<i128>()    as c_uint,
            FuncType::Pointer           => size_of::<isize>     as c_uint,
            FuncType::RefStringPtr      => size_of::<isize>     as c_uint,
            FuncType::BorrowStringPtr   => size_of::<isize>     as c_uint,
            FuncType::RefArrayPtr       => size_of::<isize>     as c_uint,
            FuncType::BorrowArrayPtr    => size_of::<isize>     as c_uint,
            FuncType::Struct(structure) => structure.size() as c_uint
        }
    }
}

impl Display for FuncType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FuncType::Auto => write!(f, "auto"),
            FuncType::Void => write!(f, "void"),

            FuncType::Int => write!(f, "int"),
            FuncType::Float => write!(f, "float"),
            FuncType::Double => write!(f, "double"),
            FuncType::LongDouble => write!(f, "long double"),
            FuncType::ISize => write!(f, "isize"),
            FuncType::USize => write!(f, "usize"),

            FuncType::S8 => write!(f, "i8"),
            FuncType::S16 => write!(f, "i16"),
            FuncType::S32 => write!(f, "i32"),
            FuncType::S64 => write!(f, "i64"),

            FuncType::U8 => write!(f, "u8"),
            FuncType::U16 => write!(f, "u16"),
            FuncType::U32 => write!(f, "u32"),
            FuncType::U64 => write!(f, "u64"),

            FuncType::F32 => write!(f, "f32"),
            FuncType::F64 => write!(f, "f64"),
            FuncType::F128 => write!(f, "f128"),

            FuncType::Pointer => write!(f, "*"),
            FuncType::RefStringPtr => write!(f, "&str"),
            FuncType::BorrowStringPtr => write!(f, "&mut str"),
            FuncType::RefArrayPtr => write!(f, "&[]"),
            FuncType::BorrowArrayPtr => write!(f, "&mut []"),

            FuncType::Struct(struct_type) => { write!(f, "{}", struct_type) }
        }
    }
}


impl FuncDescHelper {
    fn new(return_type: &FuncType, argument_types: &[FuncType]) -> Result<Self, Error> {
        unsafe {
            let mut this = Self { _boxed: Vec::new(), return_type: null_mut(), arguments_types: Box::new([]) };
            this.return_type = this.type_into_ffi_type(return_type)?;
            let mut argument_types_array = Vec::with_capacity(argument_types.len());
            for argument_type in argument_types
            { argument_types_array.push(this.type_into_ffi_type(argument_type)?); }
            this.arguments_types = argument_types_array.into_boxed_slice();
            Ok(this)
        }
    }

    unsafe fn type_into_ffi_type(&mut self, value: &FuncType) -> Result<*mut ffi_type, Error> {
        unsafe {
            Ok(
                match value {
                    FuncType::Auto => return Error::invalid_desc_from_str("Type 'auto' not supported for call"),
                    FuncType::Void => &raw mut ffi_type_void,

                    FuncType::Int => if const { size_of::<c_int>() == 32 } { &raw mut ffi_type_sint32 } else { &raw mut ffi_type_sint64 },
                    FuncType::Float => &raw mut ffi_type_float,
                    FuncType::Double => &raw mut ffi_type_double,
                    FuncType::LongDouble => &raw mut ffi_type_longdouble,
                    FuncType::ISize => if const { size_of::<isize>() == 32 } { &raw mut ffi_type_sint32 } else { &raw mut ffi_type_sint64 },
                    FuncType::USize => if const { size_of::<usize>() == 32 } { &raw mut ffi_type_uint32 } else { &raw mut ffi_type_uint64 },

                    FuncType::S8 => &raw mut ffi_type_sint8,
                    FuncType::S16 => &raw mut ffi_type_sint16,
                    FuncType::S32 => &raw mut ffi_type_sint32,
                    FuncType::S64 => &raw mut ffi_type_sint64,

                    FuncType::U8 => &raw mut ffi_type_uint8,
                    FuncType::U16 => &raw mut ffi_type_uint16,
                    FuncType::U32 => &raw mut ffi_type_uint32,
                    FuncType::U64 => &raw mut ffi_type_uint64,

                    FuncType::F32 => &raw mut ffi_type_float,
                    FuncType::F64 => &raw mut ffi_type_double,
                    FuncType::F128 => &raw mut ffi_type_longdouble,

                    FuncType::Pointer => &raw mut ffi_type_pointer,
                    FuncType::RefStringPtr => &raw mut ffi_type_pointer,
                    FuncType::BorrowStringPtr => &raw mut ffi_type_pointer,
                    FuncType::RefArrayPtr => &raw mut ffi_type_pointer,
                    FuncType::BorrowArrayPtr => &raw mut ffi_type_pointer,

                    FuncType::Struct(structure) => {
                        let fields = structure.fields();
                        let mut fields_array = Box::new(Vec::with_capacity(fields.len()));
                        for field in fields
                        { fields_array.push(self.type_into_ffi_type(&field.0)?); }
                        fields_array.push(null_mut());
                        let mut boxed = Box::new(
                            ffi_type {
                                size: 0,
                                alignment: 0,
                                type_: FFI_TYPE_STRUCT,
                                elements: fields_array.as_mut_ptr(),
                            }
                        );
                        let ptr = boxed.as_mut() as *mut ffi_type;
                        self._boxed.push((boxed, fields_array));
                        ptr
                    }
                }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(FuncType::from_str("void").unwrap(), FuncType::Void);
        assert_eq!(FuncType::from_str("float").unwrap(), FuncType::Float);
        assert_eq!(FuncType::from_str("i64").unwrap(), FuncType::S64);
        assert_eq!(FuncType::from_str("&str").unwrap(), FuncType::RefStringPtr);
        assert_eq!(FuncType::from_str("*").unwrap(), FuncType::Pointer);
    }

    #[test]
    fn test_struct() {
        assert_eq!(FuncType::from_str("[]").unwrap(), FuncType::structure(&[]));
        assert_eq!(FuncType::from_str("[f32]").unwrap(), FuncType::structure(&[FuncType::F32]));
        assert_eq!(FuncType::from_str("[f32,f64]").unwrap(), FuncType::structure(&[FuncType::F32, FuncType::F64]));
    }

    #[test]
    fn test_inner_struct() {
        assert_eq!(FuncType::from_str("[[]]").unwrap(), FuncType::structure(&[FuncType::structure(&[])]));
        assert_eq!(FuncType::from_str("[[],[]]").unwrap(), FuncType::structure(&[FuncType::structure(&[]), FuncType::structure(&[])]));
        assert_eq!(FuncType::from_str("[f32,[]]").unwrap(), FuncType::structure(&[FuncType::F32, FuncType::structure(&[])]));
        assert_eq!(FuncType::from_str("[f32,[i32,i64]]").unwrap(), FuncType::structure(&[FuncType::F32, FuncType::structure(&[FuncType::S32, FuncType::S64])]));
        assert_eq!(FuncType::from_str("[f32,[i32,[i64]]]").unwrap(), FuncType::structure(&[FuncType::F32, FuncType::structure(&[FuncType::S32, FuncType::structure(&[FuncType::S64])])]));
    }

    #[test]
    fn test_simple_desc() {
        assert_eq!(FuncDesc::from_str("(i8,i8)i16").unwrap(), FuncDesc::new(Box::new([FuncType::S8, FuncType::S8]), FuncType::S16));
    }
}