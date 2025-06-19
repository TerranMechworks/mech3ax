use crate::gamez::common::{
    read_model_array_sequential, write_model_array_sequential, MODEL_ARRAY_C_SIZE,
};
use crate::model::mw::{
    assert_model_info_zero, read_model_data, read_model_info, size_model, write_model_data,
    write_model_info, ModelMwC, MODEL_C_SIZE,
};
use log::trace;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_types::u32_to_usize;
use std::io::{Read, Write};

pub(crate) fn read_models(
    read: &mut CountingReader<impl Read>,
    end_offset: usize,
    material_count: u32,
) -> Result<(Vec<Model>, i32, i32)> {
    let model_indices = read_model_array_sequential(read)?;

    let mut prev_offset = read.offset;
    let models = model_indices
        .valid()
        .map(|index| {
            trace!("Processing model info {}/{}", index, model_indices.count);
            let wrapped = read_model_info(read)?;
            let model_offset = u32_to_usize(read.read_u32()?);
            assert_that!("model offset", prev_offset <= model_offset <= end_offset, read.prev)?;
            prev_offset = model_offset;
            Ok((wrapped, model_offset, index))
        })
        .collect::<Result<Vec<_>>>()?;

    trace!(
        "Processing {}..{} model info zeros at {}",
        model_indices.count,
        model_indices.array_size,
        read.offset
    );
    for (model_index, expected_index) in model_indices.zeros() {
        let model: ModelMwC = read.read_struct_no_log()?;
        assert_model_info_zero(&model, read.prev)
            .inspect_err(|_| trace!("{:#?} (index: {}, at {})", model, model_index, read.prev))?;

        let actual_index = read.read_i32()?;
        assert_that!("model index", actual_index == expected_index, read.prev)?;
    }
    trace!("Processed model info zeros at {}", read.offset);

    let models = models
        .into_iter()
        .map(|(wrapped, model_offset, index)| {
            trace!("Processing model data {}/{}", index, model_indices.count);
            assert_that!("model offset", read.offset == model_offset, read.offset)?;
            let model = read_model_data(read, wrapped, material_count)?;
            Ok(model)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((models, model_indices.count, model_indices.array_size))
}

pub(crate) fn write_models(
    write: &mut CountingWriter<impl Write>,
    models: &[Model],
    array_size: i32,
    offsets: &[u32],
) -> Result<()> {
    let count = assert_len!(i32, models.len(), "GameZ models")?;
    let model_indices_zero = write_model_array_sequential(write, array_size, count)?;

    let count = models.len();
    for (index, (model, offset)) in models.iter().zip(offsets.iter().copied()).enumerate() {
        trace!("Processing model info {}/{}", index, count);
        write_model_info(write, model)?;
        write.write_u32(offset)?;
    }

    trace!(
        "Processing {}..{} model info zeros at {}",
        count,
        array_size,
        write.offset
    );
    let model_zero = ModelMwC::default();
    for (_model_index, expected_index) in model_indices_zero {
        write.write_struct_no_log(&model_zero)?;
        write.write_i32(expected_index)?;
    }
    trace!("Processed model info zeros at {}", write.offset);

    for (index, model) in models.iter().enumerate() {
        trace!("Processing model data {}/{}", index, count);
        write_model_data(write, model)?;
    }

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub(crate) fn size_models(offset: u32, array_size: i32, models: &[Model]) -> (u32, Vec<u32>) {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = array_size as u32;
    let mut offset = offset + MODEL_ARRAY_C_SIZE + (MODEL_C_SIZE + U32_SIZE) * array_size;
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
