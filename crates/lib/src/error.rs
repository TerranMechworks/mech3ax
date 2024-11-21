use eyre::{eyre, Error, Result};
use std::cell::RefCell;
use std::panic::{catch_unwind, UnwindSafe};

pub const INVALID: i32 = -1;

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<Error>>> = const { RefCell::new(None) };
}

pub fn set_last_error(err: Option<Error>) {
    LAST_ERROR.with(|prev| *prev.borrow_mut() = err.map(Box::new));
}

pub fn take_last_error() -> Option<Box<Error>> {
    LAST_ERROR.with(|prev| prev.borrow_mut().take())
}

pub fn err_to_c<F>(func: F) -> i32
where
    F: FnOnce() -> Result<()> + UnwindSafe,
{
    let result = catch_unwind(|| match func() {
        Ok(()) => {
            crate::error::set_last_error(None);
            0
        }
        Err(err) => {
            crate::error::set_last_error(Some(err));
            -1
        }
    });
    match result {
        Ok(ret) => ret,
        Err(_) => {
            crate::error::set_last_error(Some(eyre!("Panicked!")));
            -2
        }
    }
}

#[no_mangle]
pub extern "C" fn last_error_length() -> i32 {
    LAST_ERROR.with(|prev| match *prev.borrow() {
        Some(ref err) => format!("{:#}", err).len() as i32 + 1,
        None => 0,
    })
}

#[no_mangle]
pub unsafe extern "C" fn last_error_message(pointer: *mut u8, length: i32) -> i32 {
    if pointer.is_null() || length < 1 {
        return INVALID;
    }

    let message = match take_last_error() {
        Some(err) => format!("{:#}", err),
        None => return INVALID,
    };

    // Cast safety: check above for >= 1, i32::MAX < usize::MAX
    let len = length as usize;
    let buffer = std::slice::from_raw_parts_mut(pointer, len);
    let count = message.len();

    if count >= buffer.len() {
        return INVALID;
    }

    std::ptr::copy_nonoverlapping(message.as_ptr(), buffer.as_mut_ptr(), count);

    buffer[count] = 0;
    // TODO: this is probably a bad idea
    count as i32
}
