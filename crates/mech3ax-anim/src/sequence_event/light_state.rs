use super::types::INPUT_NODE;
use super::utils::assert_color;
use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use mech3ax_api_types::{
    static_assert_size, AnimDef, AtNode, Color, LightState, Range, ReprSize as _, Vec3,
};
use mech3ax_common::assert::{assert_utf8, AssertionError};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::light::LightFlags;
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, bool_c, Result};
use std::io::{Read, Write};

const INPUT_NODE_INDEX: u32 = -200i32 as u32;

#[repr(C)]
struct LightStateC {
    name: [u8; 32],    // 00
    light_index: u32,  // 32
    flags: u32,        // 36
    active_state: u32, // 40
    point_source: u32, // 44
    directional: u32,  // 48
    saturated: u32,    // 52
    subdivide: u32,    // 56
    static_: u32,      // 60
    node_index: u32,   // 64
    translation: Vec3, // 68
    rotation: Vec3,    // 80
    range: Range,      // 92
    color: Color,      // 100
    ambient: f32,      // 112
    diffuse: f32,      // 116
}
static_assert_size!(LightStateC, 120);

impl ScriptObject for LightState {
    const INDEX: u8 = 4;
    const SIZE: u32 = LightStateC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("light state size", size == Self::SIZE, read.offset)?;
        let light_state: LightStateC = read.read_struct()?;

        // not sure why this information is duplicated?
        let actual_name = assert_utf8("light state name", read.prev + 0, || {
            str_from_c_padded(&light_state.name)
        })?;
        let expected_name =
            anim_def.light_from_index(light_state.light_index as usize, read.prev + 32)?;
        #[allow(unused_parens)]
        assert_that!(
            "light state name",
            &actual_name == &expected_name,
            read.prev + 32
        )?;

        let flags = LightFlags::from_bits(light_state.flags).ok_or_else(|| {
            AssertionError(format!(
                "Expected valid light state flags, but was 0x{:08X} (at {})",
                light_state.flags,
                read.prev + 36
            ))
        })?;
        // this is never set for anim.zbd
        let translation_abs = flags.contains(LightFlags::TRANSLATION_ABS);
        assert_that!(
            "light state absolute translation",
            translation_abs == false,
            read.prev + 36
        )?;

        let active_state = assert_that!("light state active state", bool light_state.active_state, read.prev + 40)?;
        // if active_state is false, then the flags should be zero. but for
        // "impact_ppc_mech", "exp_flash", "exp_flash_small", this isn't true

        // 0 = directed (never set), 1 = point source
        assert_that!(
            "light state point source",
            light_state.point_source == 1,
            read.prev + 44
        )?;

        // the values are the state, and the flag indicates whether this
        // state should be set or not

        let directional = if flags.contains(LightFlags::DIRECTIONAL) {
            let value = assert_that!("light state directional", bool light_state.directional, read.prev + 48)?;
            Some(value)
        } else {
            assert_that!(
                "light state directional",
                light_state.directional == 0,
                read.prev + 48
            )?;
            None
        };

        let saturated = if flags.contains(LightFlags::SATURATED) {
            let value =
                assert_that!("light state saturated", bool light_state.saturated, read.prev + 52)?;
            Some(value)
        } else {
            assert_that!(
                "light state saturated",
                light_state.saturated == 0,
                read.prev + 52
            )?;
            None
        };

        let subdivide = if flags.contains(LightFlags::SUBDIVIDE) {
            let value =
                assert_that!("light state subdivide", bool light_state.subdivide, read.prev + 56)?;
            Some(value)
        } else {
            assert_that!(
                "light state subdivide",
                light_state.subdivide == 0,
                read.prev + 56
            )?;
            None
        };

        let static_ = if flags.contains(LightFlags::STATIC) {
            let value =
                assert_that!("light state static", bool light_state.static_, read.prev + 60)?;
            Some(value)
        } else {
            assert_that!(
                "light state static",
                light_state.static_ == 0,
                read.prev + 60
            )?;
            None
        };

        // this is never set, so we can use the translation-only AtNode
        let rotation = flags.contains(LightFlags::ROTATION);
        assert_that!(
            "light state has rotation",
            rotation == false,
            read.prev + 36
        )?;
        assert_that!(
            "light state rotation",
            light_state.rotation == Vec3::DEFAULT,
            read.prev + 80
        )?;

        let at_node = if flags.contains(LightFlags::TRANSLATION) {
            let node = if light_state.node_index == INPUT_NODE_INDEX {
                INPUT_NODE.to_owned()
            } else {
                anim_def.node_from_index(light_state.node_index as usize, read.prev + 64)?
            };
            Some(AtNode {
                node,
                translation: light_state.translation,
            })
        } else {
            assert_that!(
                "light state at node index",
                light_state.node_index == 0,
                read.prev + 64
            )?;
            assert_that!(
                "light state translation",
                light_state.translation == Vec3::DEFAULT,
                read.prev + 68
            )?;
            None
        };

        let range = if flags.contains(LightFlags::RANGE) {
            assert_that!(
                "light state range near",
                light_state.range.min >= 0.0,
                read.prev + 92
            )?;
            assert_that!(
                "light state range far",
                light_state.range.max >= light_state.range.min,
                read.prev + 96
            )?;
            Some(light_state.range)
        } else {
            assert_that!(
                "light state range",
                light_state.range == Range::DEFAULT,
                read.prev + 92
            )?;
            None
        };

        let color = if flags.contains(LightFlags::COLOR) {
            assert_color("light state", &light_state.color, read.prev + 100)?;
            Some(light_state.color)
        } else {
            assert_that!(
                "light state color",
                light_state.color == Color::BLACK,
                read.prev + 10
            )?;
            None
        };

        let ambient = if flags.contains(LightFlags::AMBIENT) {
            assert_that!("light state ambient", 0.0 <= light_state.ambient <= 1.0, read.prev + 112)?;
            Some(light_state.ambient)
        } else {
            assert_that!(
                "light state ambient",
                light_state.ambient == 0.0,
                read.prev + 112
            )?;
            None
        };

        let diffuse = if flags.contains(LightFlags::DIFFUSE) {
            assert_that!("light state diffuse", 0.0 <= light_state.diffuse <= 1.0, read.prev + 116)?;
            Some(light_state.diffuse)
        } else {
            assert_that!(
                "light state diffuse",
                light_state.diffuse == 0.0,
                read.prev + 116
            )?;
            None
        };

        Ok(Self {
            name: expected_name,
            active_state,
            directional,
            saturated,
            subdivide,
            static_,
            at_node,
            range,
            color,
            ambient,
            diffuse,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let mut name = [0; 32];
        str_to_c_padded(&self.name, &mut name);
        let light_index = anim_def.light_to_index(&self.name)? as u32;

        let mut flags = LightFlags::empty();
        if self.directional.is_some() {
            flags |= LightFlags::DIRECTIONAL;
        }
        if self.saturated.is_some() {
            flags |= LightFlags::SATURATED;
        }
        if self.subdivide.is_some() {
            flags |= LightFlags::SUBDIVIDE;
        }
        if self.static_.is_some() {
            flags |= LightFlags::STATIC;
        }
        if self.range.is_some() {
            flags |= LightFlags::RANGE;
        }
        if self.color.is_some() {
            flags |= LightFlags::COLOR;
        }
        if self.ambient.is_some() {
            flags |= LightFlags::AMBIENT;
        }
        if self.diffuse.is_some() {
            flags |= LightFlags::DIFFUSE;
        }

        let (node_index, translation) = if let Some(at_node) = &self.at_node {
            flags |= LightFlags::TRANSLATION;
            let node_index = if at_node.node == INPUT_NODE {
                INPUT_NODE_INDEX
            } else {
                anim_def.node_to_index(&at_node.node)? as u32
            };
            (node_index, at_node.translation)
        } else {
            (0, Vec3::DEFAULT)
        };

        write.write_struct(&LightStateC {
            name,
            light_index,
            flags: flags.bits(),
            active_state: bool_c!(self.active_state),
            point_source: 1,
            directional: bool_c!(self.directional.unwrap_or(false)),
            saturated: bool_c!(self.saturated.unwrap_or(false)),
            subdivide: bool_c!(self.subdivide.unwrap_or(false)),
            static_: bool_c!(self.static_.unwrap_or(false)),
            node_index,
            translation,
            rotation: Vec3::DEFAULT,
            range: self.range.unwrap_or(Range::DEFAULT),
            color: self.color.unwrap_or(Color::BLACK),
            ambient: self.ambient.unwrap_or(0.0),
            diffuse: self.diffuse.unwrap_or(0.0),
        })?;
        Ok(())
    }
}
