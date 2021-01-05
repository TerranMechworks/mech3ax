#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op, clippy::cargo_common_metadata)]
mod errors;
mod panic;

use errors::Result;
use image::ImageOutputFormat;
use mech3rs::anim::read_anim;
use mech3rs::archive::{read_archive, Version};
use mech3rs::gamez::{read_gamez, Material, Mesh, Node};
use mech3rs::interp::read_interp;
use mech3rs::mechlib::{read_format, read_materials, read_model, read_version};
use mech3rs::messages::read_messages;
use mech3rs::motion::read_motion;
use mech3rs::reader::read_reader;
use mech3rs::textures::read_textures;
use mech3rs::CountingReader;
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufReader, Cursor};
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

type SoundCb = extern "stdcall" fn(*const c_char, *const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn sounds(filename: *const c_char, callback: SoundCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let result: Result<_> = read_archive(
            &mut input,
            |name, data, _offset| {
                let name = CString::new(name)?;
                let ptr = name.as_ptr();
                callback(ptr, data.as_ptr(), data.len());
                Ok(())
            },
            Version::One,
        );
        result?;
        Ok(())
    })
}

type InterpCb = extern "stdcall" fn(*const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn interp(filename: *const c_char, callback: InterpCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let scripts = read_interp(&mut input)?;
        let data = serde_json::to_vec(&scripts)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

// filename will be .zrd!
type ReaderCb = extern "stdcall" fn(*const c_char, *const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn reader(filename: *const c_char, callback: ReaderCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let result: Result<_> = read_archive(
            &mut input,
            |name, data, offset| {
                let mut read = CountingReader::new(Cursor::new(data));
                // translate to absolute offset
                read.offset = offset;
                let root = read_reader(&mut read)?;
                let data = serde_json::to_vec(&root)?;

                let name = CString::new(name)?;
                let ptr = name.as_ptr();
                callback(ptr, data.as_ptr(), data.len());
                Ok(())
            },
            Version::One,
        );
        result?;
        Ok(())
    })
}

type MessagesCb = extern "stdcall" fn(*const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn messages(filename: *const c_char, callback: MessagesCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = BufReader::new(File::open(filename)?);
        let messages = read_messages(&mut input, None)?;
        let data = serde_json::to_vec(&messages)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

// filename will not end in .png!
type TextureCb = extern "stdcall" fn(*const c_char, *const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn textures(filename: *const c_char, callback: TextureCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let result: Result<_> = read_textures(&mut input, |name, image| {
            let mut data = Vec::new();
            image.write_to(&mut data, ImageOutputFormat::Png)?;

            let name = CString::new(name)?;
            let ptr = name.as_ptr();
            callback(ptr, data.as_ptr(), data.len());
            Ok(())
        });
        let tex_infos = result?;
        let data = serde_json::to_vec(&tex_infos)?;

        let name = CString::new("manifest.json")?;
        let ptr = name.as_ptr();
        callback(ptr, data.as_ptr(), data.len());
        Ok(())
    })
}

// filename will not end in .json!
type MotionCb = extern "stdcall" fn(*const c_char, *const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn motion(filename: *const c_char, callback: MotionCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let result: Result<_> = read_archive(
            &mut input,
            |name, data, offset| {
                let mut read = CountingReader::new(Cursor::new(data));
                // translate to absolute offset
                read.offset = offset;
                let root = read_motion(&mut read)?;
                let data = serde_json::to_vec(&root)?;

                let name = CString::new(name)?;
                let ptr = name.as_ptr();
                callback(ptr, data.as_ptr(), data.len());
                Ok(())
            },
            Version::One,
        );
        result?;
        Ok(())
    })
}

// filename will end in .flt (except for materials)!
type MechlibCb = extern "stdcall" fn(*const c_char, *const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn mechlib(filename: *const c_char, callback: MechlibCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let result: Result<_> = read_archive(
            &mut input,
            |name, data, offset| {
                let mut read = CountingReader::new(Cursor::new(data));
                // translate to absolute offset
                read.offset = offset;

                let name_c = CString::new(name)?;
                let ptr = name_c.as_ptr();
                match name {
                    "format" => read_format(&mut read),
                    "version" => read_version(&mut read, false),
                    "materials" => {
                        let materials = read_materials(&mut read)?;
                        let data = serde_json::to_vec(&materials)?;

                        callback(ptr, data.as_ptr(), data.len());
                        Ok(())
                    }
                    _ => {
                        let root = read_model(&mut read)?;
                        let data = serde_json::to_vec(&root)?;

                        callback(ptr, data.as_ptr(), data.len());
                        Ok(())
                    }
                }?;
                Ok(())
            },
            Version::One,
        );
        result?;
        Ok(())
    })
}

type GamezCb = extern "stdcall" fn(*const u8, usize);

// the lib GameZ implementation is not serializable on purpose
#[derive(Debug, Serialize, Deserialize)]
struct GameZ {
    textures: Vec<String>,
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
}

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn gamez(filename: *const c_char, callback: GamezCb) -> *const c_char {
    err_to_c(|| {
        let filename = ptr_to_string(filename)?;
        let mut input = CountingReader::new(BufReader::new(File::open(filename)?));
        let gamez = read_gamez(&mut input)?;
        let data = serde_json::to_vec(&GameZ {
            textures: gamez.textures,
            materials: gamez.materials,
            meshes: gamez.meshes,
            nodes: gamez.nodes,
        })?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

type AnimCb = extern "stdcall" fn(*const c_char, *const u8, usize);

// filename is borrowed, return value is borrowed
#[no_mangle]
pub extern "stdcall" fn anim(filename: *const c_char, callback: AnimCb) -> *const c_char {
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
