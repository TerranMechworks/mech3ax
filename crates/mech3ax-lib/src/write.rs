use crate::buffer::CallbackBuffer;
use crate::error::err_to_c;
use crate::filename_to_string;
use anyhow::{anyhow, bail, Context, Result};
use mech3ax_api_types::{
    AnimDef, AnimMetadata, ArchiveEntry, GameZData, Material, ModelMw, Motion, Script,
    TextureManifest,
};
use mech3ax_archive::{Mode, Version};
use mech3ax_common::io_ext::CountingWriter;
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::os::raw::c_char;

fn buf_writer(ptr: *const c_char) -> Result<CountingWriter<BufWriter<File>>> {
    let path = filename_to_string(ptr)?;
    let file = File::create(&path).with_context(|| format!("Failed to create `{}`", path))?;
    Ok(CountingWriter::new(BufWriter::new(file), 0))
}

#[inline(always)]
fn buffer_callback(callback: NameBufferCb, name: &str) -> Result<Vec<u8>> {
    let mut buffer = CallbackBuffer::new();
    let ret = callback(name.as_ptr(), name.len(), &mut buffer);
    if ret != 0 {
        bail!("callback returned {} on `{}`", ret, name);
    }
    buffer
        .inner()
        .ok_or_else(|| anyhow!("callback didn't set any data on `{}`", name))
}

type NameBufferCb = extern "C" fn(*const u8, usize, *mut CallbackBuffer) -> i32;

#[no_mangle]
pub extern "C" fn write_interp(
    filename: *const c_char,
    _is_pm: i32,
    data: *const u8,
    len: usize,
) -> i32 {
    err_to_c(|| {
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
            |name, _offset| -> Result<Vec<u8>> {
                let data = buffer_callback(callback, name)?;
                transform(name, data)
            },
            version,
        )
    })
}

fn write_passthrough_transform(_name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
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
        write_passthrough_transform,
    )
}

fn write_reader_transform(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    let value: Value = serde_json::from_slice(&data)
        .with_context(|| format!("Reader data for `{}` is invalid", name))?;

    let mut buf = CountingWriter::new(Vec::new(), 0);
    mech3ax_reader::write_reader(&mut buf, &value)
        .with_context(|| format!("Failed to write reader data for `{}`", name))?;
    Ok(buf.into_inner())
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

#[no_mangle]
pub extern "C" fn write_reader_raw(
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
        write_passthrough_transform,
    )
}

fn write_motion_transform(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    let motion: Motion = serde_json::from_slice(&data)
        .with_context(|| format!("Motion data for `{}` is invalid", name))?;

    let mut buf = CountingWriter::new(Vec::new(), 0);
    mech3ax_motion::write_motion(&mut buf, &motion)
        .with_context(|| format!("Failed to write motion data for `{}`", name))?;
    Ok(buf.into_inner())
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

            let mut buf = CountingWriter::new(Vec::new(), 0);
            mech3ax_gamez::mechlib::write_materials(&mut buf, &materials)
                .context("Failed to write materials data")?;
            Ok(buf.into_inner())
        }
        original => {
            let mut model: ModelMw = serde_json::from_slice(&data)
                .with_context(|| format!("Model data for `{}` is invalid", original))?;

            let mut buf = CountingWriter::new(Vec::new(), 0);
            mech3ax_gamez::mechlib::write_model_mw(&mut buf, &mut model)
                .with_context(|| format!("Failed to write model data for `{}`", original))?;
            Ok(buf.into_inner())
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
        bail!("texture manifest is null");
    }
    let buf = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    serde_json::from_slice(buf).context("texture manifest is invalid")
}

#[no_mangle]
pub extern "C" fn write_textures(
    filename: *const c_char,
    _is_pm: i32,
    manifest_ptr: *const u8,
    manifest_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let manifest = parse_manifest(manifest_ptr, manifest_len)?;
        let mut write = buf_writer(filename)?;
        mech3ax_image::write_textures(
            &mut write,
            &manifest,
            |name| -> Result<image::DynamicImage> {
                let data = buffer_callback(callback, name)?;

                let mut reader = image::io::Reader::new(Cursor::new(data));
                reader.set_format(image::ImageFormat::Png);
                let image = reader
                    .decode()
                    .with_context(|| format!("Failed to load image data for `{}`", name))?;
                Ok(image)
            },
        )
        .context("Failed to write texture package")
    })
}

#[no_mangle]
pub extern "C" fn write_gamez(
    filename: *const c_char,
    is_pm: i32,
    data: *const u8,
    len: usize,
) -> i32 {
    err_to_c(|| {
        if is_pm != 0 {
            bail!("Pirate's Moon support for Anim isn't implemented yet");
        }
        if data.is_null() {
            bail!("gamez data is null");
        }
        let buf = unsafe { std::slice::from_raw_parts(data, len) };
        let gamez: GameZData = serde_json::from_slice(buf).context("Failed to parse GameZ data")?;
        let mut write = buf_writer(filename)?;
        mech3ax_gamez::gamez::write_gamez(&mut write, &gamez).context("Failed to write GameZ data")
    })
}

fn parse_metadata(ptr: *const u8, len: usize) -> Result<AnimMetadata> {
    if ptr.is_null() {
        bail!("anim metadata is null");
    }
    let buf = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    serde_json::from_slice(buf).context("anim metadata is invalid")
}

#[no_mangle]
pub extern "C" fn write_anim(
    filename: *const c_char,
    is_pm: i32,
    metadata_ptr: *const u8,
    metadata_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        if is_pm != 0 {
            bail!("Pirate's Moon support for Anim isn't implemented yet");
        }
        let metadata = parse_metadata(metadata_ptr, metadata_len)?;
        let mut write = buf_writer(filename)?;
        mech3ax_anim::write_anim(&mut write, &metadata, |name| -> Result<AnimDef> {
            let data = buffer_callback(callback, name)?;

            let anim_def: AnimDef = serde_json::from_slice(&data)
                .with_context(|| format!("Anim data for `{}` is invalid", name))?;
            Ok(anim_def)
        })
        .context("Failed to write Anim data")
    })
}
