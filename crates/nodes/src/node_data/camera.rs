use crate::math::cotangent;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::nodes::Camera;
use mech3ax_api_types::{Matrix, Range, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Zeros};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
pub(crate) struct CameraC {
    pub(crate) world_index: i32,       // 000
    pub(crate) window_index: i32,      // 004
    pub(crate) focus_node_xy: i32,     // 008
    pub(crate) focus_node_xz: i32,     // 012
    pub(crate) flags: u32,             // 016
    pub(crate) translation: Vec3,      // 020
    pub(crate) rotation: Vec3,         // 032
    pub(crate) world_translate: Vec3,  // 044
    pub(crate) world_rotate: Vec3,     // 056
    pub(crate) mtw_matrix: Matrix,     // 068
    pub(crate) unk104: Vec3,           // 104
    pub(crate) view_vector: Vec3,      // 116
    pub(crate) matrix: Matrix,         // 128
    pub(crate) alt_translate: Vec3,    // 164
    pub(crate) clip: Range,            // 176
    pub(crate) zero184: Zeros<24>,     // 184
    pub(crate) lod_multiplier: f32,    // 208
    pub(crate) lod_inv_sq: f32,        // 212
    pub(crate) fov_h_zoom_factor: f32, // 216
    pub(crate) fov_v_zoom_factor: f32, // 220
    pub(crate) fov_h_base: f32,        // 224
    pub(crate) fov_v_base: f32,        // 228
    pub(crate) fov: Range,             // 232
    pub(crate) fov_h_half: f32,        // 240
    pub(crate) fov_v_half: f32,        // 244
    pub(crate) one248: u32,            // 248
    pub(crate) zero252: Zeros<60>,     // 252
    pub(crate) one312: u32,            // 312
    pub(crate) zero316: Zeros<72>,     // 316
    pub(crate) one388: u32,            // 388
    pub(crate) zero392: Zeros<72>,     // 392
    pub(crate) zero464: u32,           // 464
    pub(crate) fov_h_cot: f32,         // 468
    pub(crate) fov_v_cot: f32,         // 472
    pub(crate) stride: i32,            // 476
    pub(crate) zone_set: i32,          // 480
    pub(crate) unk484: i32,            // 484
}
impl_as_bytes!(CameraC, 488);

fn assert_camera(camera: &CameraC, offset: usize) -> Result<()> {
    assert_that!("camera world index", camera.world_index == 0, offset + 0)?;
    assert_that!("camera window index", camera.window_index == 1, offset + 4)?;
    // true for mw and pm
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

    assert_that!("camera clip near z", camera.clip.min > 0.0, offset + 176)?;
    assert_that!(
        "camera clip far z",
        camera.clip.max > camera.clip.min,
        offset + 180
    )?;

    assert_that!("camera field 184", zero camera.zero184, offset + 184)?;

    assert_that!("camera LOD mul", camera.lod_multiplier == 1.0, offset + 208)?;
    assert_that!("camera LOD inv sq", camera.lod_inv_sq == 1.0, offset + 212)?;

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
    assert_that!(
        "camera FOV H base",
        camera.fov_h_base == camera.fov.min,
        offset + 224
    )?;
    assert_that!(
        "camera FOV V base",
        camera.fov_v_base == camera.fov.max,
        offset + 228
    )?;
    assert_that!(
        "camera FOV H half",
        camera.fov_h_half == camera.fov.min / 2.0,
        offset + 240
    )?;
    assert_that!(
        "camera FOV V half",
        camera.fov_v_half == camera.fov.max / 2.0,
        offset + 244
    )?;

    assert_that!("camera field 248", camera.one248 == 1, offset + 248)?;
    assert_that!("camera field 252", zero camera.zero252, offset + 252)?;

    assert_that!("camera field 312", camera.one312 == 1, offset + 312)?;
    assert_that!("camera field 316", zero camera.zero316, offset + 316)?;

    assert_that!("camera field 388", camera.one388 == 1, offset + 388)?;
    assert_that!("camera field 392", zero camera.zero392, offset + 392)?;

    assert_that!("camera field 464", camera.zero464 == 0, offset + 464)?;

    assert_that!(
        "camera FOV H tan inv",
        camera.fov_h_cot == cotangent(camera.fov_h_half),
        offset + 468
    )?;
    assert_that!(
        "camera FOV V tan inv",
        camera.fov_v_cot == cotangent(camera.fov_v_half),
        offset + 472
    )?;

    assert_that!("camera stride", camera.stride == 0, offset + 476)?;
    assert_that!("camera zone set", camera.zone_set == 0, offset + 480)?;
    assert_that!("camera field 484", camera.unk484 == -256, offset + 484)?;

    Ok(())
}

pub(crate) fn read(read: &mut CountingReader<impl Read>, data_ptr: u32) -> Result<Camera> {
    let camera: CameraC = read.read_struct()?;

    assert_camera(&camera, read.prev)?;

    Ok(Camera {
        clip: camera.clip,
        fov: camera.fov,
        focus_node_xy: camera.focus_node_xy,
        data_ptr,
    })
}

pub(crate) fn write(write: &mut CountingWriter<impl Write>, camera: &Camera) -> Result<()> {
    let fov_h_half = camera.fov.min / 2.0;
    let fov_v_half = camera.fov.max / 2.0;

    let camera = CameraC {
        world_index: 0,
        window_index: 1,
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
        clip: camera.clip,
        zero184: Zeros::new(),
        lod_multiplier: 1.0,
        lod_inv_sq: 1.0,
        fov_h_zoom_factor: 1.0,
        fov_v_zoom_factor: 1.0,
        fov_h_base: camera.fov.min,
        fov_v_base: camera.fov.max,
        fov: camera.fov,
        fov_h_half,
        fov_v_half,
        one248: 1,
        zero252: Zeros::new(),
        one312: 1,
        zero316: Zeros::new(),
        one388: 1,
        zero392: Zeros::new(),
        zero464: 0,
        fov_h_cot: cotangent(fov_h_half),
        fov_v_cot: cotangent(fov_v_half),
        stride: 0,
        zone_set: 0,
        unk484: -256,
    };
    write.write_struct(&camera)?;
    Ok(())
}

pub(crate) fn size() -> u32 {
    CameraC::SIZE
}
