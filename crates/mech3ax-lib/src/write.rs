use crate::buffer::CallbackBuffer;
use crate::error::err_to_c;
use crate::filename_to_string;
use anyhow::{bail, Context, Result};
use mech3ax_api_types::{ArchiveEntry, GameZ, Material, Model, Motion, Script, TextureManifest};
use mech3ax_archive::{Mode, Version};
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::os::raw::c_char;

fn buf_writer(ptr: *const c_char) -> Result<BufWriter<File>> {
    let path = filename_to_string(ptr)?;
    let file = File::create(&path).with_context(|| format!("Failed to create \"{}\"", &path))?;
    Ok(BufWriter::new(file))
}

type NameBufferCb = extern "C" fn(*const u8, usize, *mut CallbackBuffer) -> i32;

#[no_mangle]
pub extern "C" fn write_interp(
    filename: *const c_char,
    _is_pm: i32,
    data: *const u8,
    len: usize,
) -> i32 {
    err_to_c(move || {
        if data.is_null() {
            bail!("data is null");
        }
        let buf = unsafe { std::slice::from_raw_parts(data, len) };
        let scripts: Vec<Script> =
            serde_json::from_slice(buf).context("Failed to parse interpreter data")?;

        let mut write = buf_writer(filename)?;
        mech3ax_interp::write_interp(&mut write, &scripts)
            .context("Failed to write interpreter data")
    })
}

fn parse_entries(ptr: *const u8, len: usize) -> Result<Vec<ArchiveEntry>> {
    if ptr.is_null() {
        bail!("entries is null");
    }
    let buf = unsafe { std::slice::from_raw_parts(ptr, len) };
    serde_json::from_slice(buf).context("entries is invalid")
}

fn write_archive(
    mode: Mode,
    filename: *const c_char,
    is_pm: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
    transform: fn(&str, Vec<u8>) -> Result<Vec<u8>>,
) -> i32 {
    err_to_c(|| {
        let version = if is_pm == 0 {
            Version::One
        } else {
            Version::Two(mode)
        };
        let entries = parse_entries(entries_ptr, entries_len)?;
        let mut write = buf_writer(filename)?;
        mech3ax_archive::write_archive(
            &mut write,
            &entries,
            |name| {
                let mut buffer = CallbackBuffer::new();
                let ret = callback(name.as_ptr(), name.len(), &mut buffer);
                if ret != 0 {
                    bail!("callback returned {} on \"{}\"", ret, name);
                }
                let data = match buffer.inner() {
                    Some(data) => data,
                    None => bail!("callback didn't set any data on \"{}\"", name),
                };
                transform(name, data)
            },
            version,
        )
    })
}

fn write_sounds_transform(_name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    Ok(data)
}

#[no_mangle]
pub extern "C" fn write_sounds(
    filename: *const c_char,
    is_pm: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    write_archive(
        Mode::Sounds,
        filename,
        is_pm,
        entries_ptr,
        entries_len,
        callback,
        write_sounds_transform,
    )
}

fn write_reader_transform(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    let value: Value = serde_json::from_slice(&data)
        .with_context(|| format!("Reader data for \"{}\" is invalid", name))?;

    let mut buf = Vec::new();
    mech3ax_reader::write_reader(&mut buf, &value)
        .with_context(|| format!("Failed to write reader data for \"{}\"", name))?;
    Ok(buf)
}

#[no_mangle]
pub extern "C" fn write_reader(
    filename: *const c_char,
    is_pm: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    write_archive(
        Mode::Reader,
        filename,
        is_pm,
        entries_ptr,
        entries_len,
        callback,
        write_reader_transform,
    )
}

fn write_motion_transform(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    let motion: Motion = serde_json::from_slice(&data)
        .with_context(|| format!("Motion data for \"{}\" is invalid", name))?;

    let mut buf = Vec::new();
    mech3ax_motion::write_motion(&mut buf, &motion)
        .with_context(|| format!("Failed to write motion data for \"{}\"", name))?;
    Ok(buf)
}

#[no_mangle]
pub extern "C" fn write_motion(
    filename: *const c_char,
    is_pm: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    write_archive(
        Mode::Motion,
        filename,
        is_pm,
        entries_ptr,
        entries_len,
        callback,
        write_motion_transform,
    )
}
fn write_mechlib_transform(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    match name {
        "format" => Ok(data),
        "version" => Ok(data),
        "materials" => {
            let materials: Vec<Material> =
                serde_json::from_slice(&data).context("Materials data is invalid")?;

            let mut buf = Vec::new();
            mech3ax_gamez::mechlib::write_materials(&mut buf, &materials)
                .context("Failed to write materials data")?;
            Ok(buf)
        }
        original => {
            let mut model: Model = serde_json::from_slice(&data)
                .with_context(|| format!("Model data for \"{}\" is invalid", original))?;

            let mut buf = Vec::new();
            mech3ax_gamez::mechlib::write_model(&mut buf, &mut model)
                .with_context(|| format!("Failed to write model data for \"{}\"", original))?;
            Ok(buf)
        }
    }
}

#[no_mangle]
pub extern "C" fn write_mechlib(
    filename: *const c_char,
    is_pm: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    write_archive(
        Mode::Sounds,
        filename,
        is_pm,
        entries_ptr,
        entries_len,
        callback,
        write_mechlib_transform,
    )
}

fn parse_manifest(ptr: *const u8, len: usize) -> Result<TextureManifest> {
    if ptr.is_null() {
        bail!("manifest is null");
    }
    let buf = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    serde_json::from_slice(buf).context("texture manifest is invalid")
}

#[no_mangle]
pub extern "C" fn write_textures(
    filename: *const c_char,
    manifest_ptr: *const u8,
    manifest_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let manifest = parse_manifest(manifest_ptr, manifest_len)?;
        let mut write = buf_writer(filename)?;
        mech3ax_image::write_textures(&mut write, &manifest, |name| {
            let mut buffer = CallbackBuffer::new();
            let ret = callback(name.as_ptr(), name.len(), &mut buffer);
            if ret != 0 {
                bail!("callback returned {} on \"{}\"", ret, name);
            }
            let data = match buffer.inner() {
                Some(data) => data,
                None => bail!("callback didn't set any data on \"{}\"", name),
            };

            let mut reader = image::io::Reader::new(Cursor::new(data));
            reader.set_format(image::ImageFormat::Png);
            let image = reader
                .decode()
                .with_context(|| format!("Failed to load image data for \"{}\"", name))?;
            Ok(image)
        })
    })
}

#[no_mangle]
pub extern "C" fn write_gamez(
    filename: *const c_char,
    _is_pm: i32,
    data: *const u8,
    len: usize,
) -> i32 {
    err_to_c(move || {
        if data.is_null() {
            bail!("data is null");
        }
        let buf = unsafe { std::slice::from_raw_parts(data, len) };
        let gamez: GameZ = serde_json::from_slice(buf).context("Failed to parse GameZ data")?;
        let mut write = buf_writer(filename)?;
        mech3ax_gamez::gamez::write_gamez(&mut write, &gamez).context("Failed to write GameZ data")
    })
}
