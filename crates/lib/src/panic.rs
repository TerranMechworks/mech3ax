use std::os::raw::c_char;

#[repr(C)]
pub struct Wrapper(pub *const c_char);
unsafe impl Sync for Wrapper {}

#[unsafe(no_mangle)]
pub static PANIC: Wrapper = Wrapper(b"panic\0" as *const u8 as *const c_char);
