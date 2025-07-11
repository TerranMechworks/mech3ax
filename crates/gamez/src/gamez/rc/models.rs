use crate::gamez::common::{assert_model_array, make_model_array, ModelArrayC};
use crate::model::rc::{
    assert_model_info_zero, read_model_data, read_model_info, size_model, write_model_data,
    write_model_info, ModelRcC,
};
use log::trace;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_api_types::Count;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{chk, len, Result};
use mech3ax_types::{u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

pub(crate) fn read_models(
    read: &mut CountingReader<impl Read>,
    end_offset: usize,
    material_count: Count,
) -> Result<(Vec<Model>, Count, Count)> {
    let model_array: ModelArrayC = read.read_struct()?;
    let (count, array_size) = assert_model_array(&model_array, read.prev)?;

    let mut prev_offset = read.offset;
    let models = count
        .iter()
        .map(|index| {
            trace!("Processing model info {}/{}", index, count);
            let wrapped = read_model_info(read)?;

            let model_offset = u32_to_usize(read.read_u32()?);
            chk!(read.prev, model_offset >= prev_offset)?;
            chk!(read.prev, model_offset <= end_offset)?;

            prev_offset = model_offset;
            Ok((wrapped, model_offset, index))
        })
        .collect::<Result<Vec<_>>>()?;

    let zero_start = count.to_i32();
    let zero_end = array_size.to_i32();

    trace!(
        "Processing {}..{} model info zeros at {}",
        zero_start,
        zero_end,
        read.offset
    );
    for index in zero_start..zero_end {
        let model: ModelRcC = read.read_struct_no_log()?;
        assert_model_info_zero(&model, read.prev)
            .inspect_err(|_| trace!("{:#?} (index: {}, at {})", model, index, read.prev))?;

        let mut expected_index_next = index + 1;
        if expected_index_next >= zero_end {
            expected_index_next = -1;
        }
        let model_index_next = read.read_i32()?;
        chk!(read.prev, model_index_next == expected_index_next)?;
    }
    trace!("Processed model info zeros at {}", read.offset);

    let models = models
        .into_iter()
        .map(|(wrapped, model_offset, index)| {
            trace!("Processing model data {}/{}", index, count);
            chk!(read.offset, model_offset == read.offset)?;

            read_model_data(read, wrapped, material_count)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((models, count, array_size))
}

pub(crate) fn write_models(
    write: &mut CountingWriter<impl Write>,
    models: &[Model],
    array_size: Count,
    offsets: &[u32],
) -> Result<()> {
    let count = len!(models.len(), "GameZ models")?;
    let model_array = make_model_array(count, array_size)?;
    write.write_struct(&model_array)?;

    for (index, (model, offset)) in models.iter().zip(offsets.iter().copied()).enumerate() {
        trace!("Processing model info {}/{}", index, count);
        write_model_info(write, model, index)?;
        write.write_u32(offset)?;
    }

    let zero_start = count.to_i32();
    let zero_end = array_size.to_i32();

    trace!(
        "Processing {}..{} model info zeros at {}",
        zero_start,
        zero_end,
        write.offset
    );
    let model_zero = ModelRcC::default();
    for index in zero_start..zero_end {
        write.write_struct_no_log(&model_zero)?;

        let mut model_index_next = index + 1;
        if model_index_next >= zero_end {
            model_index_next = -1;
        }
        write.write_i32(model_index_next)?;
    }
    trace!("Processed model info zeros at {}", write.offset);

    for (index, model) in models.iter().enumerate() {
        trace!("Processing model data {}/{}", index, count);
        write_model_data(write, model, index)?;
    }

    Ok(())
}

pub(crate) fn size_models(offset: u32, array_size: Count, models: &[Model]) -> (u32, Vec<u32>) {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = array_size.to_u32();
    let mut offset = offset + ModelArrayC::SIZE + (ModelRcC::SIZE + 4) * array_size;
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
