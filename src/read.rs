use crate::error::err_to_c;
use crate::filename_to_string;
use anyhow::{bail, Context, Result};
use image::ImageOutputFormat;
use mech3rs::archive::{Mode, Version};
use mech3rs::CountingReader;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::os::raw::c_char;

fn buf_reader(ptr: *const c_char) -> Result<BufReader<File>> {
    let path = filename_to_string(ptr)?;
    let file = File::open(&path).with_context(|| format!("Failed to open \"{}\"", &path))?;
    Ok(BufReader::new(file))
}

type DataCb = extern "stdcall" fn(*const u8, usize);
type NameDataCb = extern "stdcall" fn(*const u8, usize, *const u8, usize) -> i32;

#[no_mangle]
pub extern "stdcall" fn read_interp(filename: *const c_char, _is_pm: i32, callback: DataCb) -> i32 {
    err_to_c(move || {
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let scripts =
            mech3rs::interp::read_interp(&mut read).context("Failed to read interpreter data")?;
        let data = serde_json::to_vec(&scripts)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

#[no_mangle]
pub extern "stdcall" fn read_messages(filename: *const c_char, callback: DataCb) -> i32 {
    err_to_c(|| {
        let mut read = buf_reader(filename)?;
        let messages = mech3rs::messages::read_messages(&mut read, None)
            .context("Failed to read message data")?;
        let data = serde_json::to_vec(&messages)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

fn read_archive(
    mode: Mode,
    filename: *const c_char,
    is_pm: i32,
    callback: NameDataCb,
    transform: fn(&str, Vec<u8>, u32) -> Result<Vec<u8>>,
) -> i32 {
    err_to_c(|| {
        let version = if is_pm == 0 {
            Version::One
        } else {
            Version::Two(mode)
        };
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let entries = mech3rs::archive::read_archive(
            &mut read,
            |name, data, offset| {
                let data = transform(name, data, offset)?;
                let ret = callback(name.as_ptr(), name.len(), data.as_ptr(), data.len());
                if ret != 0 {
                    bail!("callback returned {} on \"{}\"", ret, name);
                }
                Ok(())
            },
            version,
        )?;

        let name = "manifest.json";
        let data = serde_json::to_vec(&entries)?;
        let ret = callback(name.as_ptr(), name.len(), data.as_ptr(), data.len());
        if ret != 0 {
            bail!("callback returned {} on \"{}\"", ret, name);
        }
        Ok(())
    })
}

fn read_sound_transform(_name: &str, data: Vec<u8>, _offset: u32) -> Result<Vec<u8>> {
    Ok(data)
}

#[no_mangle]
pub extern "stdcall" fn read_sounds(
    filename: *const c_char,
    is_pm: i32,
    callback: NameDataCb,
) -> i32 {
    read_archive(
        Mode::Sounds,
        filename,
        is_pm,
        callback,
        read_sound_transform,
    )
}

fn read_reader_transform(name: &str, data: Vec<u8>, offset: u32) -> Result<Vec<u8>> {
    let mut read = CountingReader::new(Cursor::new(data));
    // translate to absolute offset
    read.offset = offset;
    let root = mech3rs::reader::read_reader(&mut read)
        .with_context(|| format!("Failed to read reader data for \"{}\"", name))?;
    Ok(serde_json::to_vec(&root)?)
}

// filename returned by data callback will be .zrd!
#[no_mangle]
pub extern "stdcall" fn read_reader(
    filename: *const c_char,
    is_pm: i32,
    callback: NameDataCb,
) -> i32 {
    read_archive(
        Mode::Reader,
        filename,
        is_pm,
        callback,
        read_reader_transform,
    )
}

fn read_motion_transform(name: &str, data: Vec<u8>, offset: u32) -> Result<Vec<u8>> {
    let mut read = CountingReader::new(Cursor::new(data));
    // translate to absolute offset
    read.offset = offset;
    let root = mech3rs::motion::read_motion(&mut read)
        .with_context(|| format!("Failed to read motion data for \"{}\"", name))?;
    Ok(serde_json::to_vec(&root)?)
}

// callback filename will not end in .json!
#[no_mangle]
pub extern "stdcall" fn read_motion(
    filename: *const c_char,
    is_pm: i32,
    callback: NameDataCb,
) -> i32 {
    read_archive(
        Mode::Motion,
        filename,
        is_pm,
        callback,
        read_motion_transform,
    )
}

fn read_mechlib_transform(name: &str, data: Vec<u8>, offset: u32) -> Result<Vec<u8>> {
    let mut read = CountingReader::new(Cursor::new(data));
    // translate to absolute offset
    read.offset = offset;

    match name {
        "format" => {
            mech3rs::mechlib::read_format(&mut read)
                .context("Failed to read mechlib format data")?;
            Ok(mech3rs::mechlib::FORMAT.to_le_bytes().to_vec())
        }
        "version" => {
            mech3rs::mechlib::read_version(&mut read, false)
                .context("Failed to read mechlib format data")?;
            Ok(mech3rs::mechlib::VERSION_MW.to_le_bytes().to_vec())
        }
        "materials" => {
            let materials = mech3rs::mechlib::read_materials(&mut read)
                .context("Failed to read mechlib format data")?;
            Ok(serde_json::to_vec(&materials)?)
        }
        _ => {
            let root = mech3rs::mechlib::read_model(&mut read)
                .with_context(|| format!("Failed to read model data for \"{}\"", name))?;
            Ok(serde_json::to_vec(&root)?)
        }
    }
}

// callback filename will end in .flt (except for format, version, materials)!
#[no_mangle]
pub extern "stdcall" fn read_mechlib(
    filename: *const c_char,
    is_pm: i32,
    callback: NameDataCb,
) -> i32 {
    read_archive(
        Mode::Sounds,
        filename,
        is_pm,
        callback,
        read_mechlib_transform,
    )
}

// callback filename will not end in .png! last call will be the manifest
#[no_mangle]
pub extern "stdcall" fn read_textures(filename: *const c_char, callback: NameDataCb) -> i32 {
    err_to_c(|| {
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let manifest = mech3rs::textures::read_textures(&mut read, |name, image| {
            let mut data = Vec::new();
            image
                .write_to(&mut data, ImageOutputFormat::Png)
                .with_context(|| format!("Failed to write image \"{}\"", name))?;

            let ret = callback(name.as_ptr(), name.len(), data.as_ptr(), data.len());
            if ret != 0 {
                bail!("callback returned {} on \"{}\"", ret, name);
            }
            Ok(())
        })?;

        let data = serde_json::to_vec(&manifest)?;
        let name = "manifest.json";
        let ret = callback(name.as_ptr(), name.len(), data.as_ptr(), data.len());
        if ret != 0 {
            bail!("callback returned {} on \"{}\"", ret, name);
        }
        Ok(())
    })
}

#[no_mangle]
pub extern "stdcall" fn read_gamez(filename: *const c_char, is_pm: i32, callback: DataCb) -> i32 {
    err_to_c(|| {
        if is_pm != 0 {
            bail!("Pirate's Moon support for Gamez isn't implemented yet");
        }
        let gamez = {
            let input = buf_reader(filename)?;
            let mut read = CountingReader::new(input);
            mech3rs::gamez::read_gamez(&mut read).context("Failed to read gamez data")
        }?;
        let data = serde_json::to_vec(&gamez)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

// last call will be the metadata
#[no_mangle]
pub extern "stdcall" fn read_anim(
    filename: *const c_char,
    is_pm: i32,
    callback: NameDataCb,
) -> i32 {
    err_to_c(|| {
        if is_pm != 0 {
            bail!("Pirate's Moon support for Anim isn't implemented yet");
        }
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let metadata = mech3rs::anim::read_anim(&mut read, |name, anim_def| {
            let data = serde_json::to_vec(&anim_def)?;

            let ret = callback(name.as_ptr(), name.len(), data.as_ptr(), data.len());
            if ret != 0 {
                bail!("callback returned {} on \"{}\"", ret, name);
            }
            Ok(())
        })
        .context("Failed to read anim data")?;

        let data = serde_json::to_vec(&metadata)?;
        let name = "metadata.json";
        let ret = callback(name.as_ptr(), name.len(), data.as_ptr(), data.len());
        if ret != 0 {
            bail!("callback returned {} on \"{}\"", ret, name);
        }
        Ok(())
    })
}
