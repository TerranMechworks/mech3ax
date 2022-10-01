use crate::panic;
use anyhow::Result;
use mech3ax_anim::read_anim;
use mech3ax_common::io_ext::CountingReader;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::BufReader;
use std::os::raw::c_char;
use std::panic::{catch_unwind, UnwindSafe};

fn ptr_to_string(ptr: *const c_char) -> Result<String> {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr.to_str()?.to_string())
}

fn err_to_c<F>(func: F) -> *const c_char
where
    F: FnOnce() -> Result<()> + UnwindSafe,
{
    let result = catch_unwind(|| {
        if func().is_err() {
            panic::PANIC.0
        } else {
            std::ptr::null()
        }
    });
    match result {
        Ok(ptr) => ptr,
        Err(_) => panic::PANIC.0,
    }
}

type AnimCb = extern "C" fn(*const c_char, *const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "C" fn anim(filename: *const c_char, callback: AnimCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let result: Result<_> = read_anim(&mut input, |name, anim_def| -> Result<()> {
            let data = serde_json::to_vec(&anim_def)?;

            let name = CString::new(name)?;
            let ptr = name.as_ptr();
            callback(ptr, data.as_ptr(), data.len());
            Ok(())
        });
        result?;
        Ok(())
    })
}
