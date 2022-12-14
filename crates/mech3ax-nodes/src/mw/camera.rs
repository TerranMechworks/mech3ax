use super::node::{NodeVariantMw, NodeVariantsMw};
use crate::flags::NodeBitFlags;
use crate::math::cotangent;
use crate::types::ZONE_DEFAULT;
use log::{debug, trace};
use mech3ax_api_types::nodes::mw::Camera;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::{static_assert_size, Matrix, Range, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct CameraMwC {
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
static_assert_size!(CameraMwC, 488);

const CAMERA_NAME: &str = "camera1";

pub fn assert_variants(node: NodeVariantsMw, offset: u32) -> Result<NodeVariantMw> {
    assert_that!("camera name", &node.name == CAMERA_NAME, offset + 0)?;
    assert_that!(
        "camera flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    // zero040 (40) already asserted
    assert_that!("camera field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("camera zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("camera data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("camera mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "camera area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("camera has parent", node.has_parent == false, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "camera children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    assert_that!(
        "camera bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "camera bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "camera bbox 3",
        node.unk164 == BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("camera field 196", node.unk196 == 0, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantMw::Camera {
        data_ptr: node.data_ptr,
    })
}

pub fn make_variants(camera: &Camera) -> NodeVariantsMw {
    NodeVariantsMw {
        name: CAMERA_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: camera.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    }
}

fn assert_camera(camera: CameraMwC, offset: u32) -> Result<(Range, Range)> {
    assert_that!("camera world index", camera.world_index == 0, offset + 0)?;
    assert_that!("camera window index", camera.window_index == 1, offset + 4)?;
    assert_that!(
        "camera focus node xy",
        camera.focus_node_xy == -1,
        offset + 8
    )?;
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

    assert_all_zero("camera field 184", offset + 184, &camera.zero184.0)?;

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
    assert_all_zero("camera field 252", offset + 252, &camera.zero252.0)?;

    assert_that!("camera field 312", camera.one312 == 1, offset + 312)?;
    assert_all_zero("camera field 316", offset + 316, &camera.zero316.0)?;

    assert_that!("camera field 388", camera.one388 == 1, offset + 388)?;
    assert_all_zero("camera field 392", offset + 392, &camera.zero392.0)?;

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

    Ok((camera.clip, camera.fov))
}

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Camera> {
    debug!(
        "Reading camera node data {} (mw, {}) at {}",
        index,
        CameraMwC::SIZE,
        read.offset
    );
    let camera: CameraMwC = read.read_struct()?;
    trace!("{:#?}", camera);

    let (clip, fov) = assert_camera(camera, read.prev)?;

    Ok(Camera {
        name: CAMERA_NAME.to_owned(),
        clip,
        fov,
        data_ptr,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, camera: &Camera, index: usize) -> Result<()> {
    debug!(
        "Writing camera node data {} (mw, {}) at {}",
        index,
        CameraMwC::SIZE,
        write.offset
    );

    let fov_h_half = camera.fov.min / 2.0;
    let fov_v_half = camera.fov.max / 2.0;

    let camera = CameraMwC {
        world_index: 0,
        window_index: 1,
        focus_node_xy: -1,
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
    trace!("{:#?}", camera);
    write.write_struct(&camera)?;
    Ok(())
}

pub fn size() -> u32 {
    CameraMwC::SIZE
}
