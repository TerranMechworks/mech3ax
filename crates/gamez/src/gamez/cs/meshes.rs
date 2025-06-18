use super::fixup::Fixup;
use crate::gamez::common::{read_model_array_nonseq, write_model_array_nonseq, MODEL_ARRAY_C_SIZE};
use crate::model::ng::{
    assert_model_info, assert_model_info_zero, read_model_data, size_model, write_model_data,
    write_model_info, ModelPmC, MODEL_C_SIZE,
};
use log::trace;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_types::{u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

pub(crate) fn read_models(
    read: &mut CountingReader<impl Read>,
    end_offset: usize,
    material_count: u32,
    fixup: Fixup,
) -> Result<Vec<Option<Model>>> {
    let model_array = read_model_array_nonseq(read)?;

    let mut count = 0i32;
    let mut last_index = -1i32;
    let mut prev_offset = read.offset;

    let models = model_array
        .iter()
        .map(|(model_index, expected_index)| {
            let model: ModelPmC = read.read_struct_no_log()?;

            if model.parent_count > 0 {
                trace!(
                    "Reading model info {}/{}",
                    model_index,
                    model_array.array_size
                );
                trace!("{:#?} (len: {}, at {})", model, ModelPmC::SIZE, read.prev);

                let wrapped = assert_model_info(model, read.prev)?;

                count += 1;
                last_index = expected_index;
                let model_offset = u32_to_usize(read.read_u32()?);
                assert_that!("model offset", prev_offset <= model_offset <= end_offset, read.prev)?;
                prev_offset = model_offset;

                Ok(Some((wrapped, model_offset, model_index)))
            } else {
                assert_model_info_zero(&model, read.prev).inspect_err(|_| {
                    trace!("{:#?} (index: {}, at {})", model, model_index, read.prev)
                })?;

                let expected_index = fixup.model_index_remap(expected_index);
                let actual_index = read.read_i32()?;
                assert_that!("model index", actual_index == expected_index, read.prev)?;
                Ok(None)
            }
        })
        .collect::<Result<Vec<_>>>()?;

    assert_that!("model count", count == model_array.count, read.offset)?;
    let last_index = fixup.last_index_remap(last_index);
    assert_that!(
        "model last index",
        last_index == model_array.last_index,
        read.offset
    )?;

    let models = models
        .into_iter()
        .map(|item| match item {
            Some((wrapped, offset, index)) => {
                trace!("Reading model data {}/{}", index, model_array.array_size);
                assert_that!("model offset", read.offset == offset, read.offset)?;
                let model = read_model_data(read, wrapped, material_count)?;
                Ok(Some(model))
            }
            None => Ok(None),
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(models)
}

pub(crate) fn write_models(
    write: &mut CountingWriter<impl Write>,
    models: Vec<Option<(&Model, u32)>>,
    fixup: Fixup,
) -> Result<()> {
    let array_size = assert_len!(i32, models.len(), "GameZ models")?;
    let count = models.iter().filter(|item| item.is_some()).count() as i32;
    let last_index = models
        .iter()
        .rposition(|item| item.is_some())
        .map(|i| i + 1)
        .unwrap_or(0) as i32;
    let last_index = fixup.last_index_remap(last_index);

    let model_array = write_model_array_nonseq(write, array_size, count, last_index)?;

    let model_zero = ModelPmC::default();
    for ((model_index, expected_index), item) in model_array.iter().zip(models.iter()) {
        match item {
            Some((model, offset)) => {
                trace!("Writing model info {}/{}", model_index, array_size);
                write_model_info(write, model)?;
                write.write_u32(*offset)?;
            }
            None => {
                trace!(
                    "Writing model info zero {}/{} at {}",
                    model_index,
                    array_size,
                    write.offset
                );
                write.write_struct_no_log(&model_zero)?;
                let expected_index = fixup.model_index_remap(expected_index);
                write.write_i32(expected_index)?;
            }
        }
    }

    for (index, item) in models.iter().enumerate() {
        if let Some((model, _)) = item {
            trace!("Writing model data {}/{}", index, array_size);
            write_model_data(write, model)?;
        }
    }

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub(crate) fn size_models(
    offset: u32,
    models: &[Option<Model>],
) -> (u32, Vec<Option<(&Model, u32)>>) {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = models.len() as u32;
    let mut offset = offset + MODEL_ARRAY_C_SIZE + (MODEL_C_SIZE + U32_SIZE) * array_size;
    let offsets = models
        .iter()
        .map(|model| {
            model.as_ref().map(|model| {
                let current = offset;
                offset += size_model(model);
                (model, current)
            })
        })
        .collect();
    (offset, offsets)
}
