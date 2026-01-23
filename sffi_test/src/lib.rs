#[unsafe(no_mangle)]
pub unsafe extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fadd(a: f32, b: f32) -> f32 {
    a + b
}

#[repr(C)]
pub struct Data(i32, i32, i32);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sadd(data: *mut Data) {
    unsafe { (*data).2 = (*data).0 + (*data).1; }
}