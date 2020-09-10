use image::ImageOutputFormat;
use mech3rs::anim::read_anim;
use mech3rs::archive::read_archive;
use mech3rs::gamez::read_gamez;
use mech3rs::interp::read_interp;
use mech3rs::mechlib::{read_format, read_materials, read_model, read_version};
use mech3rs::messages::read_messages;
use mech3rs::motion::read_motion;
use mech3rs::reader::read_reader;
use mech3rs::textures::read_textures;
use mech3rs::CountingReader;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};
use zip::write::{FileOptions, ZipWriter};

use crate::errors::Result;
use crate::{JsonOpts, ZipOpts};

pub(crate) fn license() -> Result<()> {
    print!(
        r#"\
mech3rs extracts assets from the MechWarrior 3 game.
Copyright (C) 2015-2020  Toby Fleming

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
"#
    );
    Ok(())
}

pub(crate) fn sounds(opts: ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let manifest: Result<_> = read_archive(&mut input, |name, data, _offset| {
        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });

    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

pub(crate) fn interp(opts: JsonOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let mut output = BufWriter::new(File::create(opts.output)?);

    let scripts = read_interp(&mut input)?;
    let data = serde_json::to_vec_pretty(&scripts)?;
    output.write_all(&data)?;
    Ok(())
}

pub(crate) fn reader(opts: ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest: Result<_> = read_archive(&mut input, |name, data, offset| {
        let name = name.replace(".zrd", ".json");
        let mut read = CountingReader::new(Cursor::new(data));
        // translate to absolute offset
        read.offset = offset;
        let root = read_reader(&mut read)?;
        let data = serde_json::to_vec_pretty(&root)?;

        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });
    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

pub(crate) fn messages(opts: JsonOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let messages = read_messages(&mut input)?;
    let data = serde_json::to_vec_pretty(&messages)?;
    output.write_all(&data)?;
    Ok(())
}

pub(crate) fn textures(opts: ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let manifest: Result<Vec<_>> = read_textures(&mut input, |name, image| {
        let name = format!("{}.png", name);
        let mut data = Vec::new();
        image.write_to(&mut data, ImageOutputFormat::Png)?;

        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

pub(crate) fn motion(opts: ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest: Result<_> = read_archive(&mut input, |name, data, offset| {
        let name = format!("{}.json", name);
        let mut read = CountingReader::new(Cursor::new(data));
        // translate to absolute offset
        read.offset = offset;
        let root = read_motion(&mut read)?;
        let data = serde_json::to_vec_pretty(&root)?;

        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });

    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

pub(crate) fn mechlib(opts: ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest: Result<_> = read_archive(&mut input, |name, data, offset| {
        let mut read = CountingReader::new(Cursor::new(data));
        // translate to absolute offset
        read.offset = offset;
        let result = match name {
            "format" => read_format(&mut read),
            "version" => read_version(&mut read),
            "materials" => {
                let materials = read_materials(&mut read)?;
                let data = serde_json::to_vec_pretty(&materials)?;

                zip.start_file("materials.json", options)?;
                zip.write_all(&data)?;
                Ok(())
            }
            other => {
                let name = other.replace(".flt", ".json");
                let root = read_model(&mut read)?;
                let data = serde_json::to_vec_pretty(&root)?;

                zip.start_file(name, options)?;
                zip.write_all(&data)?;
                Ok(())
            }
        };
        result?;
        Ok(())
    });

    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let gamez = read_gamez(&mut input)?;

    let data = serde_json::to_vec_pretty(&gamez.metadata)?;
    zip.start_file("metadata.json", options)?;
    zip.write_all(&data)?;

    let data = serde_json::to_vec_pretty(&gamez.textures)?;
    zip.start_file("textures.json", options)?;
    zip.write_all(&data)?;

    let data = serde_json::to_vec_pretty(&gamez.materials)?;
    zip.start_file("materials.json", options)?;
    zip.write_all(&data)?;

    let data = serde_json::to_vec_pretty(&gamez.meshes)?;
    zip.start_file("meshes.json", options)?;
    zip.write_all(&data)?;

    let data = serde_json::to_vec_pretty(&gamez.nodes)?;
    zip.start_file("nodes.json", options)?;
    zip.write_all(&data)?;

    zip.finish()?;
    Ok(())
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(BufReader::new(File::open(opts.input)?));
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let metadata = read_anim(&mut input, |file_name, anim_def| -> Result<()> {
        let data = serde_json::to_vec_pretty(anim_def)?;
        zip.start_file(file_name, options)?;
        zip.write_all(&data)?;
        Ok(())
    })?;

    let data = serde_json::to_vec_pretty(&metadata)?;
    zip.start_file("metadata.json", options)?;
    zip.write_all(&data)?;

    zip.finish()?;
    Ok(())
}
