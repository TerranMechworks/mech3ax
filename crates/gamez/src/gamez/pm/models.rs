use crate::gamez::common::{
    read_model_array_sequential, write_model_array_sequential, MODEL_ARRAY_C_SIZE,
};
use crate::model::pm::{
    assert_model_info_zero, make_material_refs, read_model_data, read_model_info, size_model,
    write_model_data, write_model_info, MaterialRefC, ModelPmC, MODEL_C_SIZE,
};
use log::trace;
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_api_types::Count;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, chk, len, Result};
use mech3ax_types::u32_to_usize;
use std::io::{Read, Write};

pub(crate) fn read_models(
    read: &mut CountingReader<impl Read>,
    end_offset: usize,
    material_count: Count,
) -> Result<(Vec<Model>, Count, Count)> {
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
        let model: ModelPmC = read.read_struct_no_log()?;
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
    models: &[ModelInfo],
    array_size: Count,
) -> Result<()> {
    let count = len!(models.len(), "GameZ models")?;
    let model_indices_zero = write_model_array_sequential(write, array_size, count)?;

    let count = models.len();
    for (index, model_info) in models.iter().enumerate() {
        trace!("Processing model info {}/{}", index, count);
        write_model_info(write, model_info.model, &model_info.material_refs, index)?;
        write.write_u32(model_info.offset)?;
    }

    trace!(
        "Processing {}..{} model info zeros at {}",
        count,
        array_size,
        write.offset
    );
    let model_zero = ModelPmC::default();
    for (_model_index, expected_index) in model_indices_zero {
        write.write_struct_no_log(&model_zero)?;
        write.write_i32(expected_index)?;
    }
    trace!("Processed model info zeros at {}", write.offset);

    for (index, model_info) in models.iter().enumerate() {
        trace!("Processing model data {}/{}", index, count);
        write_model_data(write, model_info.model, &model_info.material_refs, index)?;
    }

    Ok(())
}

pub(crate) struct ModelInfo<'a> {
    model: &'a Model,
    material_refs: Vec<MaterialRefC>,
    offset: u32,
}

pub(crate) fn gather_materials<'a>(
    materials: &[Material],
    models: &'a [Model],
) -> Vec<ModelInfo<'a>> {
    models
        .iter()
        .map(|model| {
            let material_refs = make_material_refs(materials, model, false);
            ModelInfo {
                model,
                material_refs,
                offset: 0,
            }
        })
        .collect()
}

pub(crate) fn size_models(offset: u32, array_size: Count, models: &mut [ModelInfo]) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = array_size.to_u32();
    let mut offset = offset + MODEL_ARRAY_C_SIZE + (MODEL_C_SIZE + 4) * array_size;
    for model_info in models {
        model_info.offset = offset;
        offset += size_model(model_info.model, &model_info.material_refs);
    }
    offset
}
