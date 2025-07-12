use eyre::{Error, Result, eyre};
use std::cell::RefCell;
use std::panic::{UnwindSafe, catch_unwind};

const INVALID: usize = 0;

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<Error>>> = const { RefCell::new(None) };
}

pub(crate) fn set_last_error(err: Option<Error>) {
    LAST_ERROR.with(|prev| *prev.borrow_mut() = err.map(Box::new));
}

pub(crate) fn take_last_error() -> Option<Box<Error>> {
    LAST_ERROR.with(|prev| prev.borrow_mut().take())
}

pub(crate) fn err_to_c<F>(func: F) -> i32
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

#[unsafe(no_mangle)]
pub extern "C" fn last_error_length() -> usize {
    LAST_ERROR.with(|prev| match *prev.borrow() {
        Some(ref err) => format!("{:#}\0", err).len(),
        None => 0,
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn last_error_message(ptr: *mut u8, len: usize) -> usize {
    if ptr.is_null() || len < 1 {
        return INVALID;
    }

    let message = match take_last_error() {
        Some(err) => format!("{:#}\0", err),
        None => return INVALID,
    };

    let count = message.len();
    if count > len {
        return INVALID;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(message.as_ptr(), ptr, count);
    }
    count.saturating_sub(1)
}
