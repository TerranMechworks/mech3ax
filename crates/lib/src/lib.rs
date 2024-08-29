#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op, clippy::cargo_common_metadata)]
mod buffer;
mod callbacks;
mod error;
mod panic;
mod read;
mod wave;
mod write;

use eyre::{bail, Result};
use mech3ax_common::GameType;
use std::ffi::CStr;
use std::os::raw::c_char;

fn filename_to_string(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        bail!("filename is null");
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr.to_str()?.to_string())
}

fn i32_to_game(game: i32) -> Result<GameType> {
    match game {
        0 => Ok(GameType::MW),
        1 => Ok(GameType::PM),
        2 => Ok(GameType::RC),
        3 => Ok(GameType::CS),
        _ => bail!("invalid game type {}", game),
    }
}
