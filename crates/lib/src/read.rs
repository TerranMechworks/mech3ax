use crate::callbacks::{DataCb, NameDataCb, WaveArchiveCb, WaveFileCb};
use crate::error::err_to_c;
use crate::wave::WaveFile;
use crate::{filename_to_string, i32_to_game};
use eyre::{bail, Context as _, Result};
use image::ImageFormat;
use mech3ax_archive::{Mode, Version};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::GameType;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::os::raw::c_char;

fn buf_reader(ptr: *const c_char) -> Result<BufReader<File>> {
    let path = filename_to_string(ptr)?;
    let file = File::open(&path).with_context(|| format!("Failed to open `{}`", &path))?;
    Ok(BufReader::new(file))
}

#[inline(always)]
fn buffer_callback(callback: NameDataCb, name: &str, data: &[u8]) -> Result<()> {
    let ret = callback(name.as_ptr(), name.len(), data.as_ptr(), data.len());
    if ret != 0 {
        bail!("callback returned {} on `{}`", ret, name);
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn read_interp(
    filename: *const c_char,
    _game_type_id: i32,
    callback: DataCb,
) -> i32 {
    err_to_c(|| {
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let scripts =
            mech3ax_interp::read_interp(&mut read).context("Failed to read interpreter data")?;
        let data = mech3ax_exchange::to_vec(&scripts)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn read_messages(
    filename: *const c_char,
    game_type_id: i32,
    callback: DataCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let mut read = buf_reader(filename)?;
        let messages = mech3ax_messages::read_messages(&mut read, game)
            .context("Failed to read message data")?;
        let data = mech3ax_exchange::to_vec(&messages)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

fn read_archive(
    version: Version,
    filename: *const c_char,
    callback: NameDataCb,
    transform: fn(&str, Vec<u8>, usize) -> Result<Vec<u8>>,
) -> Result<()> {
    let input = buf_reader(filename)?;
    let mut read = CountingReader::new(input);
    let entries = mech3ax_archive::read_archive(
        &mut read,
        |name, data, offset| {
            let data = transform(name, data, offset)?;
            buffer_callback(callback, name, &data)
        },
        version,
    )?;

    let name = "manifest.bin";
    let data = mech3ax_exchange::to_vec(&entries)?;
    buffer_callback(callback, name, &data)
}

fn read_passthrough_transform(_name: &str, data: Vec<u8>, _offset: usize) -> Result<Vec<u8>> {
    Ok(data)
}

#[no_mangle]
pub extern "C" fn read_sounds(
    filename: *const c_char,
    game_type_id: i32,
    callback: NameDataCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(Mode::Sounds),
        };
        read_archive(version, filename, callback, read_passthrough_transform)
    })
}

fn read_reader_json_transform(name: &str, data: Vec<u8>, offset: usize) -> Result<Vec<u8>> {
    let mut read = CountingReader::new(Cursor::new(data));
    // translate to absolute offset
    read.offset = offset;
    let root = mech3ax_reader::read_reader(&mut read)
        .with_context(|| format!("Failed to read reader data for `{}`", name))?;
    Ok(serde_json::to_vec(&root)?)
}

// filename returned by data callback will be .zrd!
#[no_mangle]
pub extern "C" fn read_reader_json(
    filename: *const c_char,
    game_type_id: i32,
    callback: NameDataCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(Mode::Reader),
        };
        read_archive(version, filename, callback, read_reader_json_transform)
    })
}

// filename returned by data callback will be .zrd!
#[no_mangle]
pub extern "C" fn read_reader_raw(
    filename: *const c_char,
    game_type_id: i32,
    callback: NameDataCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(Mode::Reader),
        };
        read_archive(version, filename, callback, read_passthrough_transform)
    })
}

fn read_motion_transform(name: &str, data: Vec<u8>, offset: usize) -> Result<Vec<u8>> {
    let mut read = CountingReader::new(Cursor::new(data));
    // translate to absolute offset
    read.offset = offset;
    let root = mech3ax_motion::read_motion(&mut read)
        .with_context(|| format!("Failed to read motion data for `{}`", name))?;
    Ok(mech3ax_exchange::to_vec(&root)?)
}

// callback filename will not end in .json!
#[no_mangle]
pub extern "C" fn read_motion(
    filename: *const c_char,
    game_type_id: i32,
    callback: NameDataCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW => Version::One,
            GameType::PM => Version::Two(Mode::Motion),
            GameType::RC => bail!("Recoil does not have motion"),
            GameType::CS => bail!("Crimson Skies does not have motion"),
        };
        read_archive(version, filename, callback, read_motion_transform)
    })
}

fn read_mechlib_transform_mw(name: &str, data: Vec<u8>, offset: usize) -> Result<Vec<u8>> {
    let mut read = CountingReader::new(Cursor::new(data));
    // translate to absolute offset
    read.offset = offset;

    match name {
        "format" => {
            mech3ax_gamez::mechlib::read_format(&mut read)
                .context("Failed to read mechlib format data")?;
            Ok(mech3ax_gamez::mechlib::FORMAT.to_le_bytes().to_vec())
        }
        "version" => {
            mech3ax_gamez::mechlib::read_version(&mut read, mech3ax_common::GameType::MW)
                .context("Failed to read mechlib format data")?;
            Ok(mech3ax_gamez::mechlib::VERSION_MW.to_le_bytes().to_vec())
        }
        "materials" => {
            let materials = mech3ax_gamez::mechlib::read_materials(&mut read)
                .context("Failed to read mechlib format data")?;
            Ok(mech3ax_exchange::to_vec(&materials)?)
        }
        _ => {
            let root = mech3ax_gamez::mechlib::mw::read_model(&mut read)
                .with_context(|| format!("Failed to read model data for `{}`", name))?;
            Ok(mech3ax_exchange::to_vec(&root)?)
        }
    }
}

fn read_mechlib_transform_pm(name: &str, data: Vec<u8>, offset: usize) -> Result<Vec<u8>> {
    let mut read = CountingReader::new(Cursor::new(data));
    // translate to absolute offset
    read.offset = offset;

    match name {
        "format" => {
            mech3ax_gamez::mechlib::read_format(&mut read)
                .context("Failed to read mechlib format data")?;
            Ok(mech3ax_gamez::mechlib::FORMAT.to_le_bytes().to_vec())
        }
        "version" => {
            mech3ax_gamez::mechlib::read_version(&mut read, mech3ax_common::GameType::PM)
                .context("Failed to read mechlib format data")?;
            Ok(mech3ax_gamez::mechlib::VERSION_PM.to_le_bytes().to_vec())
        }
        "materials" => {
            let materials = mech3ax_gamez::mechlib::read_materials(&mut read)
                .context("Failed to read mechlib format data")?;
            Ok(mech3ax_exchange::to_vec(&materials)?)
        }
        _ => {
            let root = mech3ax_gamez::mechlib::pm::read_model(&mut read)
                .with_context(|| format!("Failed to read model data for `{}`", name))?;
            Ok(mech3ax_exchange::to_vec(&root)?)
        }
    }
}

// callback filename will end in .flt (except for format, version, materials)!
#[no_mangle]
pub extern "C" fn read_mechlib(
    filename: *const c_char,
    game_type_id: i32,
    callback: NameDataCb,
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
            GameType::MW => read_mechlib_transform_mw,
            GameType::PM => read_mechlib_transform_pm,
            GameType::RC | GameType::CS => unreachable!(),
        };
        read_archive(version, filename, callback, transform)
    })
}

// callback filename will not end in .png! last call will be the manifest
#[no_mangle]
pub extern "C" fn read_textures(
    filename: *const c_char,
    _game_type_id: i32,
    callback: NameDataCb,
) -> i32 {
    err_to_c(|| {
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let manifest = mech3ax_image::read_textures(&mut read, |name, image| {
            let mut data = Cursor::new(Vec::new());
            image
                .write_to(&mut data, ImageFormat::Png)
                .with_context(|| format!("Failed to write image `{}`", name))?;

            buffer_callback(callback, name, data.get_ref())
        })?;

        let data = mech3ax_exchange::to_vec(&manifest)?;
        let name = "manifest.bin";
        buffer_callback(callback, name, &data)
    })
}

#[no_mangle]
pub extern "C" fn read_gamez(filename: *const c_char, game_type_id: i32, callback: DataCb) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let data = match game {
            GameType::MW => {
                let gamez = {
                    mech3ax_gamez::gamez::mw::read_gamez(&mut read)
                        .context("Failed to read gamez data")
                }?;
                mech3ax_exchange::to_vec(&gamez)?
            }
            GameType::PM => {
                let gamez = {
                    mech3ax_gamez::gamez::pm::read_gamez(&mut read)
                        .context("Failed to read gamez data")
                }?;
                mech3ax_exchange::to_vec(&gamez)?
            }
            GameType::RC => {
                let gamez = {
                    mech3ax_gamez::gamez::rc::read_gamez(&mut read)
                        .context("Failed to read gamez data")
                }?;
                mech3ax_exchange::to_vec(&gamez)?
            }
            GameType::CS => {
                let gamez = {
                    mech3ax_gamez::gamez::cs::read_gamez(&mut read)
                        .context("Failed to read gamez data")
                }?;
                mech3ax_exchange::to_vec(&gamez)?
            }
        };
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}

// last call will be the metadata
#[no_mangle]
pub extern "C" fn read_anim(
    filename: *const c_char,
    game_type_id: i32,
    callback: NameDataCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        match game {
            GameType::MW => {}
            GameType::PM => bail!("Pirate's Moon support for Anim isn't implemented yet"),
            GameType::RC => bail!("Recoil support for Anim isn't implemented yet"),
            GameType::CS => bail!("Crimson Skies support for Anim isn't implemented yet"),
        }
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let metadata = mech3ax_anim::mw::read_anim(&mut read, |name, anim_def| {
            let data = mech3ax_exchange::to_vec(&anim_def)?;

            buffer_callback(callback, name, &data)
        })
        .context("Failed to read anim data")?;

        let data = mech3ax_exchange::to_vec(&metadata)?;
        let name = "metadata.bin";
        buffer_callback(callback, name, &data)
    })
}

#[no_mangle]
pub extern "C" fn read_sounds_as_wav(
    filename: *const c_char,
    game_type_id: i32,
    callback: WaveArchiveCb,
) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        let version = match game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(Mode::Sounds),
        };
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let _entries = mech3ax_archive::read_archive(
            &mut read,
            |name, data, offset| {
                let mut read = CountingReader::new(Cursor::new(data));
                read.offset = offset;
                let wave = WaveFile::new(&mut read)?;
                let ret = callback(
                    name.as_ptr(),
                    name.len(),
                    wave.channels,
                    wave.frequency,
                    wave.samples.as_ptr(),
                    wave.samples.len(),
                );
                if ret != 0 {
                    bail!("callback returned {} on `{}`", ret, name);
                }
                Ok(())
            },
            version,
        )?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn read_sound_as_wav(filename: *const c_char, callback: WaveFileCb) -> i32 {
    err_to_c(|| {
        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let wave = WaveFile::new(&mut read)?;
        let ret = callback(
            wave.channels,
            wave.frequency,
            wave.samples.as_ptr(),
            wave.samples.len(),
        );
        if ret != 0 {
            bail!("callback returned {}", ret);
        }
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn read_zmap(filename: *const c_char, game_type_id: i32, callback: DataCb) -> i32 {
    err_to_c(|| {
        let game = i32_to_game(game_type_id)?;
        match game {
            GameType::RC => {}
            GameType::MW => bail!("MechWarrior 3 does not have zmap"),
            GameType::PM => bail!("Pirate's Moon does not have zmap"),
            GameType::CS => bail!("Crimson Skies does not have zmap"),
        }

        let input = buf_reader(filename)?;
        let mut read = CountingReader::new(input);
        let map = mech3ax_zmap::read_map(&mut read).context("Failed to read zmap data")?;
        let data = mech3ax_exchange::to_vec(&map)?;
        callback(data.as_ptr(), data.len());
        Ok(())
    })
}
