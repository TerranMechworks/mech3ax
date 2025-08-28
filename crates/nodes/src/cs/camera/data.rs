use super::info::{CAMERA_NAME, SPYGLASS_NAME};
use crate::node_data::camera::CameraC;
use mech3ax_api_types::nodes::cs::Camera;
use mech3ax_api_types::{Matrix, Range, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Zeros;
use std::io::{Read, Write};

const CLIP: Range = Range {
    min: 1.0,
    max: 5000.0,
};

fn assert_camera(camera: &CameraC, spyglass: bool, offset: usize) -> Result<()> {
    assert_that!("camera world index", camera.world_index == 0, offset + 0)?;
    let window_index = if spyglass { 4 } else { 2 };
    assert_that!(
        "camera window index",
        camera.window_index == window_index,
        offset + 4
    )?;
    // assert_that!("focus node xy", camera.focus_node_xy == -1, offset + 8)?;
    assert_that!(
        "camera focus node xz",
        camera.focus_node_xz == -1,
        offset + 12
    )?;
    assert_that!("camera flags", camera.flags == 0, offset + 16)?;
    assert_that!(
        "camera translation",
        camera.translation == Vec3::DEFAULT,
        offset + 20
    )?;
    assert_that!(
        "camera rotation",
        camera.rotation == Vec3::DEFAULT,
        offset + 32
    )?;

    assert_that!(
        "camera world translate",
        camera.world_translate == Vec3::DEFAULT,
        offset + 44
    )?;
    assert_that!(
        "camera world rotate",
        camera.world_rotate == Vec3::DEFAULT,
        offset + 56
    )?;
    assert_that!(
        "camera mtw matrix",
        camera.mtw_matrix == Matrix::EMPTY,
        offset + 68
    )?;
    assert_that!(
        "camera field 104",
        camera.unk104 == Vec3::DEFAULT,
        offset + 104
    )?;
    assert_that!(
        "camera view vector",
        camera.view_vector == Vec3::DEFAULT,
        offset + 116
    )?;
    assert_that!(
        "camera matrix",
        camera.matrix == Matrix::EMPTY,
        offset + 128
    )?;
    assert_that!(
        "camera alt translate",
        camera.alt_translate == Vec3::DEFAULT,
        offset + 164
    )?;

    assert_that!("camera clip", camera.clip == CLIP, offset + 176)?;

    assert_that!("camera field 184", zero camera.zero184, offset + 184)?;

    assert_that!("camera LOD mul", camera.lod_multiplier == 0.0, offset + 208)?;
    assert_that!("camera LOD inv sq", camera.lod_inv_sq == 0.0, offset + 212)?;

    assert_that!(
        "camera FOV H zoom factor",
        camera.fov_h_zoom_factor == 1.0,
        offset + 216
    )?;
    assert_that!(
        "camera FOV V zoom factor",
        camera.fov_v_zoom_factor == 1.0,
        offset + 220
    )?;
    assert_that!("camera FOV H base", camera.fov_h_base == 0.0, offset + 224)?;
    assert_that!("camera FOV V base", camera.fov_v_base == 0.0, offset + 228)?;
    assert_that!("camera FOV", camera.fov == Range::DEFAULT, offset + 232)?;
    assert_that!("camera FOV H half", camera.fov_h_half == 0.0, offset + 240)?;
    assert_that!("camera FOV V half", camera.fov_v_half == 0.0, offset + 244)?;

    assert_that!("camera field 248", camera.one248 == 1, offset + 248)?;
    assert_that!("camera field 252", zero camera.zero252, offset + 252)?;

    assert_that!("camera field 312", camera.one312 == 1, offset + 312)?;
    assert_that!("camera field 316", zero camera.zero316, offset + 316)?;

    assert_that!("camera field 388", camera.one388 == 1, offset + 388)?;
    assert_that!("camera field 392", zero camera.zero392, offset + 392)?;

    assert_that!("camera field 464", camera.zero464 == 0, offset + 464)?;

    assert_that!(
        "camera FOV H tan inv",
        camera.fov_h_cot == 0.0,
        offset + 468
    )?;
    assert_that!(
        "camera FOV V tan inv",
        camera.fov_v_cot == 0.0,
        offset + 472
    )?;

    assert_that!("camera stride", camera.stride == 0, offset + 476)?;
    assert_that!("camera zone set", camera.zone_set == 0, offset + 480)?;
    assert_that!("camera field 484", camera.unk484 == -256, offset + 484)?;

    Ok(())
}

pub(crate) fn read(
    read: &mut CountingReader<impl Read>,
    data_ptr: u32,
    spyglass: bool,
) -> Result<Camera> {
    let camera: CameraC = read.read_struct()?;

    assert_camera(&camera, spyglass, read.prev)?;

    let name = if spyglass { SPYGLASS_NAME } else { CAMERA_NAME };
    Ok(Camera {
        name: name.to_string(),
        focus_node_xy: camera.focus_node_xy,
        data_ptr,
    })
}

pub(crate) fn write(write: &mut CountingWriter<impl Write>, camera: &Camera) -> Result<()> {
    let window_index = if camera.name == SPYGLASS_NAME { 4 } else { 2 };

    let camera = CameraC {
        world_index: 0,
        window_index,
        focus_node_xy: camera.focus_node_xy,
        focus_node_xz: -1,
        flags: 0,
        translation: Vec3::DEFAULT,
        rotation: Vec3::DEFAULT,
        world_translate: Vec3::DEFAULT,
        world_rotate: Vec3::DEFAULT,
        mtw_matrix: Matrix::EMPTY,
        unk104: Vec3::DEFAULT,
        view_vector: Vec3::DEFAULT,
        matrix: Matrix::EMPTY,
        alt_translate: Vec3::DEFAULT,
        clip: CLIP,
        zero184: Zeros::new(),
        lod_multiplier: 0.0,
        lod_inv_sq: 0.0,
        fov_h_zoom_factor: 1.0,
        fov_v_zoom_factor: 1.0,
        fov_h_base: 0.0,
        fov_v_base: 0.0,
        fov: Range::DEFAULT,
        fov_h_half: 0.0,
        fov_v_half: 0.0,
        one248: 1,
        zero252: Zeros::new(),
        one312: 1,
        zero316: Zeros::new(),
        one388: 1,
        zero392: Zeros::new(),
        zero464: 0,
        fov_h_cot: 0.0,
        fov_v_cot: 0.0,
        stride: 0,
        zone_set: 0,
        unk484: -256,
    };
    write.write_struct(&camera)?;
    Ok(())
}
