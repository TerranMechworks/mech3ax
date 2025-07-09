use crate::buffer::CallbackBuffer;
use crate::callbacks::NameBufferCb;
use crate::error::err_to_c;
use crate::{filename_to_string, i32_to_game};
use eyre::{bail, eyre, Context as _, Result};
use mech3ax_api_types::anim::AnimMetadata;
use mech3ax_api_types::archive::ArchiveEntry;
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::gamez::{GameZ, MechlibModel};
use mech3ax_api_types::image::TextureManifest;
use mech3ax_api_types::interp::Script;
use mech3ax_api_types::motion::Motion;
use mech3ax_api_types::zmap::Zmap;
use mech3ax_archive::{Mode, Version};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::GameType;
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
        .ok_or_else(|| eyre!("callback didn't set any data on `{}`", name))
}

#[no_mangle]
pub extern "C" fn write_interp(
    filename: *const c_char,
    _game_type_id: i32,
    data: *const u8,
    len: usize,
) -> i32 {
    err_to_c(|| {
        if data.is_null() {
            bail!("data is null");
        }
        let buf = unsafe { std::slice::from_raw_parts(data, len) };
        let scripts: Vec<Script> =
            mech3ax_exchange::from_slice(buf).context("Failed to parse interpreter data")?;

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
    mech3ax_exchange::from_slice(buf).context("entries is invalid")
}

fn write_archive(
    version: Version,
    filename: *const c_char,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
    transform: fn(&str, Vec<u8>) -> Result<Vec<u8>>,
) -> Result<()> {
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
}

fn write_passthrough_transform(_name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    Ok(data)
}

#[no_mangle]
pub extern "C" fn write_sounds(
    filename: *const c_char,
    game_type_id: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(Mode::Sounds),
        };
        write_archive(
            version,
            filename,
            entries_ptr,
            entries_len,
            callback,
            write_passthrough_transform,
        )
    })
}

fn write_reader_json_transform(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    let value: Value = serde_json::from_slice(&data)
        .with_context(|| format!("Reader data for `{}` is invalid", name))?;

    let mut buf = CountingWriter::new(Vec::new(), 0);
    mech3ax_reader::write_reader(&mut buf, &value)
        .with_context(|| format!("Failed to write reader data for `{}`", name))?;
    Ok(buf.into_inner())
}

#[no_mangle]
pub extern "C" fn write_reader_json(
    filename: *const c_char,
    game_type_id: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(Mode::Reader),
        };
        write_archive(
            version,
            filename,
            entries_ptr,
            entries_len,
            callback,
            write_reader_json_transform,
        )
    })
}

#[no_mangle]
pub extern "C" fn write_reader_raw(
    filename: *const c_char,
    game_type_id: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(Mode::Reader),
        };
        write_archive(
            version,
            filename,
            entries_ptr,
            entries_len,
            callback,
            write_passthrough_transform,
        )
    })
}

fn write_motion_transform(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    let motion: Motion = mech3ax_exchange::from_slice(&data)
        .with_context(|| format!("Motion data for `{}` is invalid", name))?;

    let mut buf = CountingWriter::new(Vec::new(), 0);
    mech3ax_motion::write_motion(&mut buf, &motion)
        .with_context(|| format!("Failed to write motion data for `{}`", name))?;
    Ok(buf.into_inner())
}

#[no_mangle]
pub extern "C" fn write_motion(
    filename: *const c_char,
    game_type_id: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW => Version::One,
            GameType::PM => Version::Two(Mode::Motion),
            GameType::RC => bail!("Recoil does not have motion"),
            GameType::CS => bail!("Crimson Skies does not have motion"),
        };
        write_archive(
            version,
            filename,
            entries_ptr,
            entries_len,
            callback,
            write_motion_transform,
        )
    })
}

fn write_mechlib_transform_mw(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    match name {
        "format" => Ok(data),
        "version" => Ok(data),
        "materials" => {
            let materials: Vec<Material> =
                mech3ax_exchange::from_slice(&data).context("Materials data is invalid")?;

            let mut buf = CountingWriter::new(Vec::new(), 0);
            mech3ax_gamez::mechlib::write_materials(&mut buf, &materials)
                .context("Failed to write materials data")?;
            Ok(buf.into_inner())
        }
        original => {
            let model: MechlibModel = mech3ax_exchange::from_slice(&data)
                .with_context(|| format!("Model data for `{}` is invalid", original))?;

            let mut buf = CountingWriter::new(Vec::new(), 0);
            mech3ax_gamez::mechlib::mw::write_model(&mut buf, &model)
                .with_context(|| format!("Failed to write model data for `{}`", original))?;
            Ok(buf.into_inner())
        }
    }
}

fn write_mechlib_transform_pm(name: &str, data: Vec<u8>) -> Result<Vec<u8>> {
    match name {
        "format" => Ok(data),
        "version" => Ok(data),
        "materials" => {
            let materials: Vec<Material> =
                mech3ax_exchange::from_slice(&data).context("Materials data is invalid")?;

            let mut buf = CountingWriter::new(Vec::new(), 0);
            mech3ax_gamez::mechlib::write_materials(&mut buf, &materials)
                .context("Failed to write materials data")?;
            Ok(buf.into_inner())
        }
        original => {
            let model: MechlibModel = mech3ax_exchange::from_slice(&data)
                .with_context(|| format!("Model data for `{}` is invalid", original))?;

            let mut buf = CountingWriter::new(Vec::new(), 0);
            mech3ax_gamez::mechlib::pm::write_model(&mut buf, &model)
                .with_context(|| format!("Failed to write model data for `{}`", original))?;
            Ok(buf.into_inner())
        }
    }
}

#[no_mangle]
pub extern "C" fn write_mechlib(
    filename: *const c_char,
    game_type_id: i32,
    entries_ptr: *const u8,
    entries_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW => Version::One,
            GameType::PM => Version::Two(Mode::Sounds),
            GameType::RC => bail!("Recoil does not have mechlib"),
            GameType::CS => bail!("Crimson Skies does not have mechlib"),
        };
        let transform = match game {
            GameType::MW => write_mechlib_transform_mw,
            GameType::PM => write_mechlib_transform_pm,
            GameType::RC | GameType::CS => unreachable!(),
        };
        write_archive(
            version,
            filename,
            entries_ptr,
            entries_len,
            callback,
            transform,
        )
    })
}

fn parse_manifest(ptr: *const u8, len: usize) -> Result<TextureManifest> {
    if ptr.is_null() {
        bail!("texture manifest is null");
    }
    let buf = unsafe { std::slice::from_raw_parts(ptr, len) };
    mech3ax_exchange::from_slice(buf).context("texture manifest is invalid")
}

#[no_mangle]
pub extern "C" fn write_textures(
    filename: *const c_char,
    _game_type_id: i32,
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

                let mut reader = image::ImageReader::new(Cursor::new(data));
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
    game_type_id: i32,
    data: *const u8,
    len: usize,
) -> i32 {
    err_to_c(|| {
        if data.is_null() {
            bail!("gamez data is null");
        }
        let game = i32_to_game(game_type_id)?;
        let buf = unsafe { std::slice::from_raw_parts(data, len) };
        let gamez: GameZ =
            mech3ax_exchange::from_slice(buf).context("Failed to parse GameZ data")?;

        let mut write = buf_writer(filename)?;
        match game {
            GameType::MW => mech3ax_gamez::gamez::mw::write_gamez(&mut write, &gamez)
                .context("Failed to write GameZ data"),
            GameType::PM => mech3ax_gamez::gamez::pm::write_gamez(&mut write, &gamez)
                .context("Failed to write GameZ data"),
            GameType::RC => mech3ax_gamez::gamez::rc::write_gamez(&mut write, &gamez)
                .context("Failed to write GameZ data"),
            GameType::CS => bail!("Crimson Skies support for GameZ isn't implemented any more"),
        }
    })
}

fn parse_metadata(ptr: *const u8, len: usize) -> Result<AnimMetadata> {
    if ptr.is_null() {
        bail!("anim metadata is null");
    }
    let buf = unsafe { std::slice::from_raw_parts(ptr, len) };
    mech3ax_exchange::from_slice(buf).context("anim metadata is invalid")
}

#[no_mangle]
pub extern "C" fn write_anim(
    filename: *const c_char,
    game_type_id: i32,
    metadata_ptr: *const u8,
    metadata_len: usize,
    callback: NameBufferCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        match game {
            GameType::MW => {}
            GameType::PM => bail!("Pirate's Moon support for Anim isn't implemented yet"),
            GameType::RC => bail!("Recoil support for Anim isn't implemented yet"),
            GameType::CS => bail!("Crimson Skies support for Anim isn't implemented yet"),
        }
        let metadata = parse_metadata(metadata_ptr, metadata_len)?;
        let mut write = buf_writer(filename)?;

        let load_item =
            |item_name: mech3ax_anim::LoadItemName<'_>| -> Result<mech3ax_anim::LoadItem> {
                let name = item_name.name();
                let data = buffer_callback(callback, name)?;

                match item_name {
                    mech3ax_anim::LoadItemName::AnimDef(name) => {
                        mech3ax_exchange::from_slice(&data)
                            .map(mech3ax_anim::LoadItem::AnimDef)
                            .with_context(|| format!("Anim def for `{}` is invalid", name))
                    }
                    mech3ax_anim::LoadItemName::SiScript(name) => {
                        mech3ax_exchange::from_slice(&data)
                            .map(mech3ax_anim::LoadItem::SiScript)
                            .with_context(|| format!("SI script for `{}` is invalid", name))
                    }
                }
            };

        mech3ax_anim::mw::write_anim(&mut write, &metadata, load_item)
            .context("Failed to write Anim data")
    })
}

#[no_mangle]
pub extern "C" fn write_zmap(
    filename: *const c_char,
    game_type_id: i32,
    data: *const u8,
    len: usize,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        match game {
            GameType::RC => {}
            GameType::MW => bail!("MechWarrior 3 does not have zmap"),
            GameType::PM => bail!("Pirate's Moon does not have zmap"),
            GameType::CS => bail!("Crimson Skies does not have zmap"),
        }

        if data.is_null() {
            bail!("data is null");
        }
        let buf = unsafe { std::slice::from_raw_parts(data, len) };
        let map: Zmap = mech3ax_exchange::from_slice(buf).context("Failed to parse zmap data")?;

        let mut write = buf_writer(filename)?;
        mech3ax_zmap::write_map(&mut write, &map).context("Failed to write zmap data")
    })
}
