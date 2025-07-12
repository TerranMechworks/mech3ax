use super::super::EventPm;
use crate::types::{AnimDefLookup as _, INPUT_NODE_NAME, Idx32, index};
use crate::utils::assert_color;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{AtNode, LightState, LightType, Translate};
use mech3ax_api_types::{Color, Range, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Ascii, Bool32, Maybe, bitflags, impl_as_bytes};
use std::io::{Read, Write};

bitflags! {
    pub struct LightFlags: u32 {
        // This flag never occurs in animation definitions, but does in GameZ
        const TRANSLATE_ABS = 1 << 0;   // 0x0001
        const AT_NODE = 1 << 1;         // 0x0002
        const ORIENTATION = 1 << 2;     // 0x0004
        const RANGE = 1 << 3;           // 0x0008
        const COLOR = 1 << 4;           // 0x0010
        const AMBIENT = 1 << 5;         // 0x0020
        const DIFFUSE = 1 << 6;         // 0x0040
        const DIRECTIONAL = 1 << 7;     // 0x0080
        const SATURATED = 1 << 8;       // 0x0100
        const SUBDIVIDE = 1 << 9;       // 0x0200
        const STATIC = 1 << 10;         // 0x0400
        const AMBIENT_COLOR = 1 << 11;  // 0x0800
        const BICOLORED = 1 << 12;      // 0x1000
        const LIGHTMAP = 1 << 13;       // 0x2000
    }
}

type Flags = Maybe<u32, LightFlags>;
type Type = Maybe<u32, LightType>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LightStatePmC {
    light_name: Ascii<32>, // 000
    light_index: Idx32,    // 032
    flags: Flags,          // 36
    active_state: Bool32,  // 040
    ty_: Type,             // 044
    directional: Bool32,   // 048
    saturated: Bool32,     // 052
    subdivide: Bool32,     // 056
    lightmap: Bool32,      // 060
    static_: Bool32,       // 064
    bicolored: Bool32,     // 068
    node_index: Idx32,     // 072
    translate: Vec3,       // 076
    orientation: Vec3,     // 088
    range: Range,          // 100
    color: Color,          // 108
    ambient_color: Color,  // 120
    ambient: f32,          // 132
    diffuse: f32,          // 136
}
impl_as_bytes!(LightStatePmC, 140);

fn assert_flag_and_bool(
    name: &str,
    flags: LightFlags,
    flag: LightFlags,
    value: Bool32,
    offset: usize,
) -> Result<Option<bool>> {
    if flags.contains(flag) {
        let value = assert_that!(name, bool value, offset)?;
        Ok(Some(value))
    } else {
        assert_that!(name, value == Bool32::FALSE, offset)?;
        Ok(None)
    }
}

impl EventPm for LightState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(LightStatePmC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("light state size", size == LightStatePmC::SIZE, read.offset)?;
        let state: LightStatePmC = read.read_struct()?;

        let name = assert_utf8("light state name", read.prev + 0, || {
            state.light_name.to_str_padded()
        })?;
        let expected_name = anim_def.light_from_index(state.light_index, read.prev + 32)?;
        assert_that!("light state name", name == expected_name, read.prev + 32)?;

        let flags = assert_that!("light state flags", flags state.flags, read.prev + 36)?;

        let active_state =
            assert_that!("light state active state", bool state.active_state, read.prev + 40)?;
        // if active_state is false, then the flags should be zero
        if !active_state {
            assert_that!(
                "light state flags",
                flags == LightFlags::empty(),
                read.prev + 36
            )?;
        }

        let light_type = assert_that!("light state type", enum state.ty_, read.prev + 44)?;

        let directional = assert_flag_and_bool(
            "light state directional",
            flags,
            LightFlags::DIRECTIONAL,
            state.directional,
            read.prev + 48,
        )?;
        let saturated = assert_flag_and_bool(
            "light state saturated",
            flags,
            LightFlags::SATURATED,
            state.saturated,
            read.prev + 52,
        )?;
        let subdivide = assert_flag_and_bool(
            "light state subdivide",
            flags,
            LightFlags::SUBDIVIDE,
            state.subdivide,
            read.prev + 56,
        )?;
        let lightmap = assert_flag_and_bool(
            "light state lightmap",
            flags,
            LightFlags::LIGHTMAP,
            state.lightmap,
            read.prev + 60,
        )?;
        let static_ = assert_flag_and_bool(
            "light state static",
            flags,
            LightFlags::STATIC,
            state.static_,
            read.prev + 64,
        )?;
        let bicolored = assert_flag_and_bool(
            "light state bicolored",
            flags,
            LightFlags::BICOLORED,
            state.bicolored,
            read.prev + 68,
        )?;

        let translate = if flags.contains(LightFlags::AT_NODE) {
            let name = if state.node_index == index!(input) {
                INPUT_NODE_NAME.to_string()
            } else {
                anim_def.node_from_index(state.node_index, read.prev + 72)?
            };
            Some(Translate::AtNode(AtNode {
                name,
                pos: state.translate,
            }))
        } else if flags.contains(LightFlags::TRANSLATE_ABS) {
            assert_that!(
                "light state at node index",
                state.node_index == index!(0),
                read.prev + 72
            )?;
            Some(Translate::Absolute(state.translate))
        } else {
            assert_that!(
                "light state at node index",
                state.node_index == index!(0),
                read.prev + 72
            )?;
            assert_that!(
                "light state translation",
                state.translate == Vec3::DEFAULT,
                read.prev + 76
            )?;
            None
        };

        // this is never set
        let orientation = if flags.contains(LightFlags::ORIENTATION) {
            Some(state.orientation)
        } else {
            assert_that!(
                "light state orientation",
                state.orientation == Vec3::DEFAULT,
                read.prev + 88
            )?;
            None
        };

        let range = if flags.contains(LightFlags::RANGE) {
            assert_that!(
                "light state range near",
                state.range.min >= 0.0,
                read.prev + 100
            )?;
            assert_that!(
                "light state range far",
                state.range.max >= state.range.min,
                read.prev + 104
            )?;
            Some(state.range)
        } else {
            assert_that!(
                "light state range",
                state.range == Range::DEFAULT,
                read.prev + 100
            )?;
            None
        };

        let color = if flags.contains(LightFlags::COLOR) {
            assert_color!("light state color", state.color, read.prev + 108)?;
            Some(state.color)
        } else {
            assert_that!(
                "light state color",
                state.color == Color::BLACK,
                read.prev + 108
            )?;
            None
        };

        let ambient_color = if flags.contains(LightFlags::AMBIENT_COLOR) {
            assert_color!(
                "light state ambient color",
                state.ambient_color,
                read.prev + 120
            )?;
            Some(state.ambient_color)
        } else {
            assert_that!(
                "light state ambient color",
                state.ambient_color == Color::BLACK,
                read.prev + 120
            )?;
            None
        };

        let ambient = if flags.contains(LightFlags::AMBIENT) {
            assert_that!("light state ambient", 0.0 <= state.ambient <= 1.0, read.prev + 132)?;
            Some(state.ambient)
        } else {
            assert_that!("light state ambient", state.ambient == 0.0, read.prev + 132)?;
            None
        };

        let diffuse = if flags.contains(LightFlags::DIFFUSE) {
            assert_that!("light state diffuse", 0.0 <= state.diffuse <= 1.0, read.prev + 136)?;
            Some(state.diffuse)
        } else {
            assert_that!("light state diffuse", state.diffuse == 0.0, read.prev + 136)?;
            None
        };

        Ok(Self {
            name,
            active_state,
            type_: light_type,
            directional,
            saturated,
            subdivide,
            lightmap,
            static_,
            bicolored,
            translate,
            orientation,
            range,
            color,
            ambient_color,
            ambient,
            diffuse,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let light_name = Ascii::from_str_padded(&self.name);
        let light_index = anim_def.light_to_index(&self.name)?;

        let mut flags = LightFlags::empty();
        let mut node_index = index!(0);
        let translate = match &self.translate {
            Some(Translate::AtNode(AtNode { name, pos })) => {
                flags |= LightFlags::AT_NODE;
                if name == INPUT_NODE_NAME {
                    node_index = index!(input);
                } else {
                    node_index = anim_def.node_to_index(name)?;
                }
                *pos
            }
            Some(Translate::Absolute(pos)) => {
                flags |= LightFlags::TRANSLATE_ABS;
                *pos
            }
            None => Vec3::DEFAULT,
        };
        if self.orientation.is_some() {
            flags |= LightFlags::ORIENTATION;
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
        if self.ambient_color.is_some() {
            flags |= LightFlags::AMBIENT_COLOR;
        }
        if self.bicolored.is_some() {
            flags |= LightFlags::BICOLORED;
        }
        if self.lightmap.is_some() {
            flags |= LightFlags::LIGHTMAP;
        }

        let light_state = LightStatePmC {
            light_name,
            light_index,
            flags: flags.maybe(),
            active_state: self.active_state.into(),
            ty_: self.type_.maybe(),
            directional: self.directional.unwrap_or(false).into(),
            saturated: self.saturated.unwrap_or(false).into(),
            subdivide: self.subdivide.unwrap_or(false).into(),
            lightmap: self.lightmap.unwrap_or(false).into(),
            static_: self.static_.unwrap_or(false).into(),
            bicolored: self.bicolored.unwrap_or(false).into(),
            node_index,
            translate,
            orientation: self.orientation.unwrap_or(Vec3::DEFAULT),
            range: self.range.unwrap_or(Range::DEFAULT),
            color: self.color.unwrap_or(Color::BLACK),
            ambient_color: self.ambient_color.unwrap_or(Color::BLACK),
            ambient: self.ambient.unwrap_or(0.0),
            diffuse: self.diffuse.unwrap_or(0.0),
        };
        write.write_struct(&light_state)?;
        Ok(())
    }
}
