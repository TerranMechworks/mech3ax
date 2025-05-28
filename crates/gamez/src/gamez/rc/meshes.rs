use crate::gamez::common::{
    read_meshes_info_sequential, write_meshes_info_sequential, MESHES_INFO_C_SIZE,
};
use crate::mesh::rc::{
    assert_model_info_zero, read_model_data, read_model_info, size_model, write_model_data,
    write_model_info, ModelRcC, MODEL_C_SIZE,
};
use log::trace;
use mech3ax_api_types::gamez::mesh::ModelRc;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_types::u32_to_usize;
use std::io::{Read, Write};

const MODEL_ARRAY_SIZE: i32 = 6000;

pub(crate) fn read_models(
    read: &mut CountingReader<impl Read>,
    end_offset: usize,
    material_count: u32,
) -> Result<(Vec<ModelRc>, i32)> {
    let model_indices = read_meshes_info_sequential(read)?;
    assert_that!(
        "models array size",
        model_indices.array_size == MODEL_ARRAY_SIZE,
        read.offset
    )?;

    let mut prev_offset = read.offset;
    let models = model_indices
        .valid()
        .map(|model_index| {
            trace!("Reading model info {}/{}", model_index, model_indices.count);
            let wrapped = read_model_info(read)?;
            let model_offset = u32_to_usize(read.read_u32()?);
            assert_that!("mesh offset", prev_offset <= model_offset <= end_offset, read.prev)?;
            prev_offset = model_offset;
            Ok((wrapped, model_offset, model_index))
        })
        .collect::<Result<Vec<_>>>()?;

    trace!(
        "Reading {}..{} model info zeros at {}",
        model_indices.count,
        model_indices.array_size,
        read.offset
    );
    for (model_index, expected_index) in model_indices.zeros() {
        let model: ModelRcC = read.read_struct_no_log()?;
        assert_model_info_zero(&model, read.prev)
            .inspect_err(|_| trace!("{:#?} (index: {}, at {})", model, model_index, read.prev))?;

        let actual_index = read.read_i32()?;
        assert_that!("model index", actual_index == expected_index, read.prev)?;
    }
    trace!("Read model info zeros at {}", read.offset);

    let models = models
        .into_iter()
        .map(|(wrapped, model_offset, model_index)| {
            trace!("Reading model data {}/{}", model_index, model_indices.count);
            assert_that!("model offset", read.offset == model_offset, read.offset)?;
            let model = read_model_data(read, wrapped, material_count)?;
            Ok(model)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((models, model_indices.count))
}

pub(crate) fn write_models(
    write: &mut CountingWriter<impl Write>,
    models: &[ModelRc],
    offsets: &[u32],
) -> Result<()> {
    let count = assert_len!(i32, models.len(), "GameZ models")?;
    let model_indices_zero = write_meshes_info_sequential(write, MODEL_ARRAY_SIZE, count)?;

    let count = models.len();
    for (model_index, (model, offset)) in models.iter().zip(offsets.iter().copied()).enumerate() {
        trace!("Writing model info {}/{}", model_index, count);
        write_model_info(write, model)?;
        write.write_u32(offset)?;
    }

    trace!(
        "Writing {}..{} model info zeros at {}",
        count,
        MODEL_ARRAY_SIZE,
        write.offset
    );
    let model_zero = ModelRcC::default();
    for (_model_index, expected_index) in model_indices_zero {
        write.write_struct_no_log(&model_zero)?;
        write.write_i32(expected_index)?;
    }
    trace!("Wrote model info zeros at {}", write.offset);

    for (model_index, model) in models.iter().enumerate() {
        trace!("Writing model data {}/{}", model_index, count);
        write_model_data(write, model)?;
    }

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub(crate) fn size_models(offset: u32, models: &[ModelRc]) -> (u32, Vec<u32>) {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = MODEL_ARRAY_SIZE as u32;
    let mut offset = offset + MESHES_INFO_C_SIZE + (MODEL_C_SIZE + U32_SIZE) * array_size;
    let offsets = models
        .iter()
        .map(|model| {
            let current = offset;
            offset += size_model(model);
            current
        })
        .collect();
    (offset, offsets)
}
