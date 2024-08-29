use crate::buffer::CallbackBuffer;

pub(crate) type DataCb = extern "C" fn(*const u8, usize);
pub(crate) type NameDataCb = extern "C" fn(*const u8, usize, *const u8, usize) -> i32;
pub(crate) type NameBufferCb = extern "C" fn(*const u8, usize, *mut CallbackBuffer) -> i32;

pub(crate) type WaveArchiveCb = extern "C" fn(*const u8, usize, i32, i32, *const f32, usize) -> i32;
pub(crate) type WaveFileCb = extern "C" fn(i32, i32, *const f32, usize) -> i32;
