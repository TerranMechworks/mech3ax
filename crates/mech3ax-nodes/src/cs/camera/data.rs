use super::info::{CAMERA_NAME, SPYGLASS_NAME};
use log::{debug, trace};
use mech3ax_api_types::nodes::cs::Camera;
use mech3ax_api_types::{static_assert_size, Matrix, Range, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct CameraCsC {
    world_index: i32,       // 000
    window_index: i32,      // 004
    focus_node_xy: i32,     // 008
    focus_node_xz: i32,     // 012
    flags: u32,             // 016
    translation: Vec3,      // 020
    rotation: Vec3,         // 032
    world_translate: Vec3,  // 044
    world_rotate: Vec3,     // 056
    mtw_matrix: Matrix,     // 068
    unk104: Vec3,           // 104
    view_vector: Vec3,      // 116
    matrix: Matrix,         // 128
    alt_translate: Vec3,    // 164
    clip: Range,            // 176
    zero184: Zeros<24>,     // 184
    lod_multiplier: f32,    // 208
    lod_inv_sq: f32,        // 212
    fov_h_zoom_factor: f32, // 216
    fov_v_zoom_factor: f32, // 220
    fov_h_base: f32,        // 224
    fov_v_base: f32,        // 228
    fov: Range,             // 232
    fov_h_half: f32,        // 240
    fov_v_half: f32,        // 244
    one248: u32,            // 248
    zero252: Zeros<60>,     // 252
    one312: u32,            // 312
    zero316: Zeros<72>,     // 316
    one388: u32,            // 388
    zero392: Zeros<72>,     // 392
    zero464: u32,           // 464
    fov_h_cot: f32,         // 468
    fov_v_cot: f32,         // 472
    stride: i32,            // 476
    zone_set: i32,          // 480
    unk484: i32,            // 484
}
static_assert_size!(CameraCsC, 488);

const CLIP: Range = Range {
    min: 1.0,
    max: 5000.0,
};

fn assert_camera(camera: &CameraCsC, spyglass: bool, offset: u32) -> Result<()> {
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

    assert_all_zero("camera field 184", offset + 184, &camera.zero184.0)?;

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
    assert_all_zero("camera field 252", offset + 252, &camera.zero252.0)?;

    assert_that!("camera field 312", camera.one312 == 1, offset + 312)?;
    assert_all_zero("camera field 316", offset + 316, &camera.zero316.0)?;

    assert_that!("camera field 388", camera.one388 == 1, offset + 388)?;
    assert_all_zero("camera field 392", offset + 392, &camera.zero392.0)?;

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

pub fn read(
    read: &mut CountingReader<impl Read>,
    data_ptr: u32,
    spyglass: bool,
    index: usize,
) -> Result<Camera> {
    debug!(
        "Reading camera node data {} (cs, {}) at {}",
        index,
        CameraCsC::SIZE,
        read.offset
    );
    let camera: CameraCsC = read.read_struct()?;
    trace!("{:#?}", camera);

    assert_camera(&camera, spyglass, read.prev)?;

    let name = if spyglass { SPYGLASS_NAME } else { CAMERA_NAME };
    Ok(Camera {
        name: name.to_string(),
        focus_node_xy: camera.focus_node_xy,
        data_ptr,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, camera: &Camera, index: usize) -> Result<()> {
    debug!(
        "Writing camera node data {} (cs, {}) at {}",
        index,
        CameraCsC::SIZE,
        write.offset
    );

    let window_index = if camera.name == SPYGLASS_NAME { 4 } else { 2 };

    let camera = CameraCsC {
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
    trace!("{:#?}", camera);
    write.write_struct(&camera)?;
    Ok(())
}
