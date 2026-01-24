use sffi::adapter::Adapter;
use sffi::api::adapter::{sffi_adapter_call, sffi_adapter_free, sffi_adapter_get, sffi_adapter_parse, sffi_adapter_set};
use sffi::api::error::{sffi_error_free, sffi_error_msg};
use sffi::api::interface::sffi_func_call;
use sffi::api::library::{sffi_lib_close, sffi_lib_func, sffi_lib_open, sffi_lib_symbol};
use sffi::api::structure::{sffi_struct_calloc, sffi_struct_free, sffi_struct_get_raw, sffi_struct_set_raw, sffi_struct_type_alloc, sffi_struct_type_free};
use sffi::interface::FuncHandle;
use sffi::library::LibHandle;
use sffi::structure::StructType;
use std::ffi::{c_char, c_void, CStr};
use std::mem::transmute;
use std::ptr::{null, null_mut};

fn main() {
    unsafe {
        let mut lib = LibHandle::open("/home/dmn/Workspace/projects/sffi/sffi_test/target/debug/libsffi_test.so").unwrap();

        let symbol = lib.symbol("puts").unwrap();
        let symbol = transmute::<_, unsafe extern "C" fn(text: *const c_char)>(symbol);
        symbol(transmute(b"(symbol call)\t Hello, Symbol!\0"));

        let func = lib.func("puts", "(&str)void").unwrap();
        func.call(null_mut(), &mut [transmute(&b"(func call)\t Hello, Function!\0")]);

        let func = lib.func("add", "(i32,i32)i32").unwrap();
        let mut result = 0i32;
        let a = 12i32;
        let b = 21i32;
        func.call(transmute(&mut result), &mut [transmute(&a), transmute(&b)]);
        println!("(func call)\t {}", result);

        let func = lib.func("add", "(i32,i32)i32").unwrap();
        let adapter = Adapter::from_str("(f32,f32)f32").unwrap();
        let mut result = 0f32;
        let a = 1.444f32;
        let b = 2.333f32;
        adapter.call(func, transmute(&mut result), &mut [transmute(&a), transmute(&b)]).unwrap();
        println!("(adapter call)\t {}", result);

        let func = lib.func("sadd", "([i32,i32,i32])void").unwrap();
        let struct_type = StructType::from_str("[i32,i32,i32]").unwrap();
        let structure = struct_type.calloc();
        struct_type.set_raw(structure, 0, &4i32 as *const i32 as *const c_void);
        struct_type.set_raw(structure, 1, &5i32 as *const i32 as *const c_void);
        func.call(null_mut(), &mut [transmute(&structure)]);
        let mut result = 0i32;
        struct_type.get_raw(structure, 2, &mut result as *mut i32 as *mut c_void);
        StructType::free(structure);
        println!("(struct raw)\t {}", result);

        let func = lib.func("sadd", "([i32,i32,i32])void").unwrap();
        let struct_type = StructType::from_str("[i32,i32,i32]").unwrap();
        let adapter = Adapter::from_str("[f32,f32,f32]").unwrap();
        let structure = struct_type.calloc();
        adapter.set(&struct_type, structure, 0, &6.12f32 as *const f32 as *mut c_void).unwrap();
        adapter.set(&struct_type, structure, 1, &4.21f32 as *const f32 as *mut c_void).unwrap();
        func.call(null_mut(), &mut [transmute(&structure)]);
        let mut result = 0f32;
        adapter.get(&struct_type, structure, 2, &mut result as *mut f32 as *mut c_void).unwrap();
        StructType::free(structure);
        println!("(struct adapter) {}", result);
    }

    println!();

    unsafe {
        macro_rules! pie {
            ($expr:expr) => {
                let err = $expr;
                if !err.is_null() {
                    let str = CStr::from_ptr(sffi_error_msg(err)).to_str().unwrap();
                    sffi_error_free(err);
                    panic!("{}", str);
                }
            };
        }

        let mut lib: *mut LibHandle = null_mut();
        pie!(sffi_lib_open(&mut lib, b"/home/dmn/Workspace/projects/sffi/sffi_test/target/debug/libsffi_test.so\0".as_ptr() as *const c_char));

        let mut symbol: *const c_void = null();
        pie!(sffi_lib_symbol(&mut symbol, lib, b"puts\0".as_ptr() as *const c_char));
        let symbol = transmute::<_, unsafe extern "C" fn(text: *const c_char)>(symbol);
        symbol(b"(symbol)\t Hello, Symbol!\0".as_ptr() as *const c_char);

        let mut func: *const FuncHandle = null_mut();
        pie!(sffi_lib_func(&mut func, lib, b"puts\0".as_ptr() as *const c_char, "(&str)void\0".as_ptr() as *const c_char));
        let text = b"(func call)\t Hello, Function!\0";
        let args = [&text];
        sffi_func_call(func, null_mut(), transmute(&args));

        let mut func: *const FuncHandle = null();
        pie!(sffi_lib_func(&mut func, lib, b"add\0".as_ptr() as *const c_char, "(i32,i32)i32\0".as_ptr() as *const c_char));
        let mut result = 0i32;
        let a = 12i32;
        let b = 21i32;
        let args = [&a, &b];
        sffi_func_call(func, transmute(&mut result), transmute(&args));
        println!("(func call)\t {}", result);

        let mut func: *const FuncHandle = null();
        pie!(sffi_lib_func(&mut func, lib, b"add\0".as_ptr() as *const c_char, b"(i32,i32)i32\0".as_ptr() as *const c_char));
        let mut adapter: *mut Adapter = null_mut();
        pie!(sffi_adapter_parse(&mut adapter, b"(f32,f32)f32\0".as_ptr() as *const c_char));
        let mut result = 0f32;
        let a = 1.444f32;
        let b = 2.333f32;
        let args = [&a, &b];
        sffi_adapter_call(adapter, func, transmute(&mut result), 2, transmute(&args));
        sffi_adapter_free(adapter);
        println!("(adapter call)\t {}", result);

        let mut func: *const FuncHandle = null();
        pie!(sffi_lib_func(&mut func, lib, b"sadd\0".as_ptr() as *const c_char, b"([i32,i32,i32])void\0".as_ptr() as *const c_char));
        let mut struct_type: *mut StructType = null_mut();
        pie!(sffi_struct_type_alloc(&mut struct_type, b"[i32,i32,i32]\0".as_ptr() as *const c_char));
        let structure = sffi_struct_calloc(struct_type);
        sffi_struct_set_raw(struct_type, structure, 0, &4i32 as *const i32 as *mut c_void);
        sffi_struct_set_raw(struct_type, structure, 1, &5i32 as *const i32 as *mut c_void);
        sffi_func_call(func, null_mut(), transmute(&[&structure]));
        let mut result = 0i32;
        sffi_struct_get_raw(struct_type, structure, 2, &mut result as *mut i32 as *mut c_void);
        sffi_struct_free(structure);
        sffi_struct_type_free(struct_type);
        println!("(struct raw)\t {}", result);

        let mut func: *const FuncHandle = null();
        pie!(sffi_lib_func(&mut func, lib, b"sadd\0".as_ptr() as *const c_char, b"([i32,i32,i32])void\0".as_ptr() as *const c_char));
        let mut struct_type: *mut StructType = null_mut();
        pie!(sffi_struct_type_alloc(&mut struct_type, b"[i32,i32,i32]\0".as_ptr() as *const c_char));
        let mut adapter: *mut Adapter = null_mut();
        pie!(sffi_adapter_parse(&mut adapter, b"[f32,f32,f32]\0".as_ptr() as *const c_char));
        let structure = sffi_struct_calloc(struct_type);
        pie!(sffi_adapter_set(adapter, struct_type, structure, 0, &6.12f32 as *const f32 as *const c_void));
        pie!(sffi_adapter_set(adapter, struct_type, structure, 1, &4.21f32 as *const f32 as *mut c_void));
        sffi_func_call(func, null_mut(), transmute(&[&structure]));
        let mut result = 0f32;
        pie!(sffi_adapter_get(adapter, struct_type, structure, 2, &mut result as *mut f32 as *mut c_void));
        sffi_struct_free(structure);
        sffi_adapter_free(adapter);
        sffi_struct_type_free(struct_type);
        println!("(struct adapter) {}", result);

        sffi_lib_close(lib);
    }
}
