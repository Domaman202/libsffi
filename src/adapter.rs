use crate::interface::{FuncDesc, FuncHandle, FuncType};
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, c_double, c_float, c_int, c_uint, c_void};
use std::ops::{Deref, DerefMut};
use std::ptr::{read, write};
use crate::error::Error;
use crate::internal::{ends_with, starts_with};
use crate::structure::StructType;

pub struct Adapter {
    target: FuncDesc
}

struct SafeAlloc {
    layout: Layout,
    value: *mut c_void
}

impl Adapter {
    pub fn new(target: FuncDesc) -> Self {
        Self { target }
    }

    pub fn from_str(str: &str) -> Result<Self, Error> {
        let str = str.trim_start();
        if starts_with(str, '[') {
            if ends_with(str, ']') {
                let str = format!("({})void", &str[1..str.len() - 1]);
                Ok(Self::new(FuncDesc::from_str(str.as_str())?))
            } else {
                Error::invalid_desc_from_str("Invalid descriptor end")
            }
        } else {
            Ok(Self::new(FuncDesc::from_str(str)?))
        }
    }

    pub fn call(&self, func: *const FuncHandle, result: *mut c_void, arguments: &mut [*mut c_void]) -> Result<(), Error> {
        unsafe { self._call(func, result, arguments.len() as c_uint, arguments.as_mut_ptr()) }
    }

    pub(crate) unsafe fn _call(&self, func: *const FuncHandle, result: *mut c_void, argc: c_uint, argv: *mut *mut c_void) -> Result<(), Error> {
        unsafe {
            self._call_check_arguments(func, argc)?;
            
            let func_return_type = (*func).desc().return_type();
            let func_return = SafeAlloc::alloc(func_return_type);

            let argc = argc as usize;
            let mut func_arguments_safe = Vec::with_capacity(argc);
            let mut func_arguments = Vec::with_capacity(argc);
            let target_arguments_types = self.target.argument_types();
            let func_arguments_types = (*func).desc().argument_types();
            for i in 0..argc {
                let func_type = &func_arguments_types[i];
                let allocation = SafeAlloc::alloc(func_type);
                Self::_call_cast_type(&target_arguments_types[i], func_type, *argv.add(i), *allocation)?;
                func_arguments.push(*allocation);
                func_arguments_safe.push(allocation);
            }
            
            (*func)._call(*func_return, func_arguments.as_mut_ptr());

            Self::_call_cast_type(func_return_type, self.target.return_type(), *func_return, result)?;
            
            Ok(())
        }
    }

    pub fn set(&self, struct_type: &StructType, structure: *mut c_void, index: c_uint, avalue: *const c_void) -> Result<(), Error> {
        unsafe { self._set(struct_type, structure, index, avalue) }
    }

    pub(crate) unsafe fn _set(&self, struct_type: *const StructType, structure: *mut c_void, index: c_uint, avalue: *const c_void) -> Result<(), Error> {
        unsafe {
            self._access_check_arguments(&*struct_type, index)?;
            let from_type = self.target.argument_types().get_unchecked(index as usize);
            let (into_type, offset) = (*struct_type).fields().get_unchecked(index as usize);
            Self::_call_cast_type(from_type, into_type, avalue, structure.byte_offset(*offset as isize))
        }
    }

    pub fn get(&self, struct_type: &StructType, structure: *const c_void, index: c_uint, rvalue: *mut c_void) -> Result<(), Error> {
        unsafe { self._get(struct_type, structure, index, rvalue) }
    }

    pub(crate) unsafe fn _get(&self, struct_type: *const StructType, structure: *const c_void, index: c_uint, rvalue: *mut c_void) -> Result<(), Error> {
        unsafe {
            self._access_check_arguments(&*struct_type, index)?;
            let into_type = self.target.argument_types().get_unchecked(index as usize);
            let (from_type, offset) = (*struct_type).fields().get_unchecked(index as usize);
            Self::_call_cast_type(from_type, into_type, structure.byte_offset(*offset as isize), rvalue)
        }
    }

    fn _call_cast_type(from_type: &FuncType, into_type: &FuncType, from_addr: *const c_void, into_addr: *mut c_void) -> Result<(), Error> { // todo: fix long double
        unsafe {
            if from_type.is_auto() {
                match into_type {
                    FuncType::Auto => unreachable!(),
                    FuncType::Void => {}

                    FuncType::Int               => write::<c_int>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::Float             => write::<c_float> (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::Double            => write::<c_double>(into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::LongDouble        => write::<i128>    (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::ISize             => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::USize             => write::<usize>   (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::S8                => write::<i8>      (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::S16               => write::<i16>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::S32               => write::<i32>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::S64               => write::<i64>     (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::U8                => write::<u8>      (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::U16               => write::<u16>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::U32               => write::<u32>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::U64               => write::<u64>     (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::F32               => write::<f32>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::F64               => write::<f64>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::F128              => write::<i128>    (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::Pointer           => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::RefStringPtr      => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::BorrowStringPtr   => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::RefArrayPtr       => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::BorrowArrayPtr    => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::Struct(_)         => todo!()
                }
                return Ok(())
            }

            if into_type.is_auto() {
                match from_type {
                    FuncType::Auto => unreachable!(),
                    FuncType::Void => {}

                    FuncType::Int               => write::<c_int>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::Float             => write::<c_float> (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::Double            => write::<c_double>(into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::LongDouble        => write::<i128>    (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::ISize             => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::USize             => write::<usize>   (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::S8                => write::<i8>      (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::S16               => write::<i16>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::S32               => write::<i32>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::S64               => write::<i64>     (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::U8                => write::<u8>      (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::U16               => write::<u16>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::U32               => write::<u32>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::U64               => write::<u64>     (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::F32               => write::<f32>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::F64               => write::<f64>     (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::F128              => write::<i128>    (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::Pointer           => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::RefStringPtr      => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::BorrowStringPtr   => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::RefArrayPtr       => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),
                    FuncType::BorrowArrayPtr    => write::<isize>   (into_addr as *mut _, read(from_addr as *const _)),

                    FuncType::Struct(_)         => todo!()
                }
                return Ok(())
            }

            if from_type.is_ptr() && into_type.is_ptr() {
                if from_type.is_ref_str() && into_type.is_borrow_str() {
                    unsafe extern "C" { fn strdup(str: *const c_char) -> *mut c_char; }
                    write::<*const c_char>(into_addr as *mut _, strdup(read(from_addr as *const _)));
                } else {
                    write::<isize>(into_addr as *mut _, read(from_addr as *const _));
                }
                return Ok(())
            }

            let (value, flag): (i64, bool) =
                match from_type {
                    FuncType::Int   => (read::<c_int>   (from_addr as *const _) as _, true),
                    FuncType::ISize => (read::<isize>   (from_addr as *const _) as _, true),

                    FuncType::S8    => (read::<i8>      (from_addr as *const _) as _, true),
                    FuncType::S16   => (read::<i16>     (from_addr as *const _) as _, true),
                    FuncType::S32   => (read::<i32>     (from_addr as *const _) as _, true),
                    FuncType::S64   => (read::<i64>     (from_addr as *const _) as _, true),
                    _ => (0, false)
                };
            if flag {
                match into_type {
                    FuncType::Int       => write::<c_int>       (into_addr as *mut _, value as _),
                    FuncType::Float     => write::<c_float>     (into_addr as *mut _, value as _),
                    FuncType::Double    => write::<c_double>    (into_addr as *mut _, value as _),
                    FuncType::LongDouble=> write::<i128>        (into_addr as *mut _, value as _),
                    FuncType::ISize     => write::<isize>       (into_addr as *mut _, value as _),
                    FuncType::USize     => write::<usize>       (into_addr as *mut _, value as _),

                    FuncType::S8        => write::<i8>          (into_addr as *mut _, value as _),
                    FuncType::S16       => write::<i16>         (into_addr as *mut _, value as _),
                    FuncType::S32       => write::<i32>         (into_addr as *mut _, value as _),
                    FuncType::S64       => write::<i64>         (into_addr as *mut _, value as _),

                    FuncType::U8        => write::<u8>          (into_addr as *mut _, value as _),
                    FuncType::U16       => write::<u16>         (into_addr as *mut _, value as _),
                    FuncType::U32       => write::<u32>         (into_addr as *mut _, value as _),
                    FuncType::U64       => write::<u64>         (into_addr as *mut _, value as _),

                    FuncType::F32       => write::<f32>         (into_addr as *mut _, value as _),
                    FuncType::F64       => write::<f64>         (into_addr as *mut _, value as _),
                    FuncType::F128      => write::<i128>        (into_addr as *mut _, value as _),

                    _ => return Error::invalid_cast_from_string(format!("Invalid cast from '{}' into '{}'", from_type, into_type)),
                }
                return Ok(())
            }

            let (value, flag): (u64, bool) =
                match from_type {
                    FuncType::USize => (read::<usize>   (from_addr as *const _) as _, true),

                    FuncType::U8    => (read::<u8>      (from_addr as *const _) as _, true),
                    FuncType::U16   => (read::<u16>     (from_addr as *const _) as _, true),
                    FuncType::U32   => (read::<u32>     (from_addr as *const _) as _, true),
                    FuncType::U64   => (read::<u64>     (from_addr as *const _) as _, true),
                    _ => (0, false)
                };
            if flag {
                match into_type {
                    FuncType::Int       => write::<c_int>       (into_addr as *mut _, value as _),
                    FuncType::Float     => write::<c_float>     (into_addr as *mut _, value as _),
                    FuncType::Double    => write::<c_double>    (into_addr as *mut _, value as _),
                    FuncType::LongDouble=> write::<i128>        (into_addr as *mut _, value as _),
                    FuncType::ISize     => write::<isize>       (into_addr as *mut _, value as _),
                    FuncType::USize     => write::<usize>       (into_addr as *mut _, value as _),

                    FuncType::S8        => write::<i8>          (into_addr as *mut _, value as _),
                    FuncType::S16       => write::<i16>         (into_addr as *mut _, value as _),
                    FuncType::S32       => write::<i32>         (into_addr as *mut _, value as _),
                    FuncType::S64       => write::<i64>         (into_addr as *mut _, value as _),

                    FuncType::U8        => write::<u8>          (into_addr as *mut _, value as _),
                    FuncType::U16       => write::<u16>         (into_addr as *mut _, value as _),
                    FuncType::U32       => write::<u32>         (into_addr as *mut _, value as _),
                    FuncType::U64       => write::<u64>         (into_addr as *mut _, value as _),

                    FuncType::F32       => write::<f32>         (into_addr as *mut _, value as _),
                    FuncType::F64       => write::<f64>         (into_addr as *mut _, value as _),
                    FuncType::F128      => write::<i128>        (into_addr as *mut _, value as _),

                    _ => return Error::invalid_cast_from_string(format!("Invalid cast from '{}' into '{}'", from_type, into_type)),
                }
                return Ok(())
            }

            let (value, flag): (f64, bool) =
                match from_type {
                    FuncType::Float     => (read::<c_float> (from_addr as *const _) as _, true),
                    FuncType::Double    => (read::<c_double>(from_addr as *const _) as _, true),
                    FuncType::LongDouble=> (read::<i128>    (from_addr as *const _) as _, true),

                    FuncType::F32       => (read::<f32>     (from_addr as *const _) as _, true),
                    FuncType::F64       => (read::<f64>     (from_addr as *const _) as _, true),
                    FuncType::F128      => (read::<i128>    (from_addr as *const _) as _, true),
                    _ => (0.0, false)
                };
            if flag {
                match into_type {
                    FuncType::Int       => write::<c_int>       (into_addr as *mut _, value as _),
                    FuncType::Float     => write::<c_float>     (into_addr as *mut _, value as _),
                    FuncType::Double    => write::<c_double>    (into_addr as *mut _, value as _),
                    FuncType::LongDouble=> write::<i128>        (into_addr as *mut _, value as _),
                    FuncType::ISize     => write::<isize>       (into_addr as *mut _, value as _),
                    FuncType::USize     => write::<usize>       (into_addr as *mut _, value as _),

                    FuncType::S8        => write::<i8>          (into_addr as *mut _, value as _),
                    FuncType::S16       => write::<i16>         (into_addr as *mut _, value as _),
                    FuncType::S32       => write::<i32>         (into_addr as *mut _, value as _),
                    FuncType::S64       => write::<i64>         (into_addr as *mut _, value as _),

                    FuncType::U8        => write::<u8>          (into_addr as *mut _, value as _),
                    FuncType::U16       => write::<u16>         (into_addr as *mut _, value as _),
                    FuncType::U32       => write::<u32>         (into_addr as *mut _, value as _),
                    FuncType::U64       => write::<u64>         (into_addr as *mut _, value as _),

                    FuncType::F32       => write::<f32>         (into_addr as *mut _, value as _),
                    FuncType::F64       => write::<f64>         (into_addr as *mut _, value as _),
                    FuncType::F128      => write::<i128>        (into_addr as *mut _, value as _),

                    _ => return Error::invalid_cast_from_string(format!("Invalid cast from '{}' into '{}'", from_type, into_type)),
                }
                return Ok(())
            }

            Error::invalid_cast_from_string(format!("Cast from '{}' into '{}' unsupported", from_type, into_type))
        }
    }

    fn _call_alloc_type(r#type: &FuncType) -> *mut c_void {
        unsafe { alloc(Self::_call_calc_type_layout(r#type)) as *mut c_void }
    }

    fn _call_free_type(ptr: *mut c_void, r#type: &FuncType) {
        unsafe { dealloc(ptr as *mut u8, Self::_call_calc_type_layout(r#type)); }
    }

    fn _call_calc_type_layout(r#type: &FuncType) -> Layout {
        match r#type {
            FuncType::Auto              => unreachable!(),
            FuncType::Void              => unsafe { Layout::array::<u8>(0).unwrap_unchecked() },

            FuncType::Int               => Layout::new::<c_int>(),
            FuncType::Float             => Layout::new::<c_float>(),
            FuncType::Double            => Layout::new::<c_double>(),
            FuncType::LongDouble        => Layout::new::<u128>(),
            FuncType::ISize             => Layout::new::<isize>(),
            FuncType::USize             => Layout::new::<usize>(),

            FuncType::S8                => Layout::new::<i8>(),
            FuncType::S16               => Layout::new::<i16>(),
            FuncType::S32               => Layout::new::<i32>(),
            FuncType::S64               => Layout::new::<i64>(),

            FuncType::U8                => Layout::new::<u8>(),
            FuncType::U16               => Layout::new::<u16>(),
            FuncType::U32               => Layout::new::<u32>(),
            FuncType::U64               => Layout::new::<u64>(),

            FuncType::F32               => Layout::new::<f32>(),
            FuncType::F64               => Layout::new::<f64>(),
            FuncType::F128              => Layout::new::<i128>(),

            FuncType::Pointer           => Layout::new::<*mut c_void>(),
            FuncType::RefStringPtr      => Layout::new::<*mut c_void>(),
            FuncType::BorrowStringPtr   => Layout::new::<*mut c_void>(),
            FuncType::RefArrayPtr       => Layout::new::<*mut c_void>(),
            FuncType::BorrowArrayPtr    => Layout::new::<*mut c_void>(),

            FuncType::Struct(_)         => todo!()
        }
    }

    fn _call_check_arguments(&self, func: *const FuncHandle, argc: c_uint) -> Result<(), Error> {
        let target_argc = self.target.argument_types().len();
        let func_argc = unsafe { (*func).desc().argument_types().len() };
        if target_argc != func_argc { return Error::invalid_args_from_string(format!("Function invalid arguments count ({} / {})", func_argc, target_argc)); }
        let accepted_argc = argc as usize;
        if target_argc != accepted_argc { return Error::invalid_args_from_string(format!("Accepted invalid arguments count ({} / {})", accepted_argc, target_argc)) }
        Ok(())
    }

    fn _access_check_arguments(&self, struct_type: &StructType, index: c_uint) -> Result<(), Error> {
        let target_count = self.target.argument_types().len();
        if target_count <= index as usize { return Error::invalid_args_from_string(format!("Invalid index ({} / {})", index, target_count)); }
        let struct_count = struct_type.fields().len();
        if target_count != struct_count { return Error::invalid_args_from_string(format!("Structure invalid fields count ({} / {})", struct_count, target_count)); }
        Ok(())
    }
}

impl SafeAlloc {
    fn alloc(r#type: &FuncType) -> Self {
        unsafe {
            let layout = Adapter::_call_calc_type_layout(r#type);
            Self {
                layout,
                value: alloc(layout) as *mut c_void,
            }
        }
    }
}

impl Drop for SafeAlloc {
    fn drop(&mut self) {
        unsafe { dealloc(self.value as *mut u8, self.layout) }
    }
}

impl Deref for SafeAlloc {
    type Target = *mut c_void;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for SafeAlloc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}