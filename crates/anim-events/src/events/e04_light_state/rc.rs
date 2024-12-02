use super::super::EventRc;
use crate::types::{index, AnimDefLookup as _, Idx32, INPUT_NODE_NAME};
use crate::utils::assert_color;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{AtNode, LightState, LightType, Translate};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Color, Range, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Ascii, Bool32, Maybe};
use std::io::{Read, Write};

bitflags! {
    pub struct LightFlags: u32 {
        const TRANSLATE_ABS = 1 << 0;   // 0x0001
        const AT_NODE = 1 << 1;         // 0x0002
        const ORIENTATION = 1 << 2;     // 0x0004
        const RANGE = 1 << 3;           // 0x0008
        const COLOR = 1 << 4;           // 0x0010
        const AMBIENT = 1 << 5;         // 0x0020
        const DIFFUSE = 1 << 6;         // 0x0040
        const DIRECTIONAL = 1 << 7;     // 0x0080
        const SATURATED = 1 << 8;       // 0x0100
    }
}

type Flags = Maybe<u32, LightFlags>;
type Type = Maybe<u32, LightType>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LightStateRcC {
    light_name: Ascii<32>, // 00
    light_index: Idx32,    // 032
    flags: Flags,          // 36
    active_state: Bool32,  // 40
    ty_: Type,             // 44
    directional: Bool32,   // 48
    saturated: Bool32,     // 52
    node_index: Idx32,     // 56
    translate: Vec3,       // 60
    orientation: Vec3,     // 72
    range: Range,          // 84
    color: Color,          // 92
    ambient: f32,          // 104
    diffuse: f32,          // 108
}
impl_as_bytes!(LightStateRcC, 112);

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

impl EventRc for LightState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(LightStateRcC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("light state size", size == LightStateRcC::SIZE, read.offset)?;
        let state: LightStateRcC = read.read_struct()?;

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

        let translate = if flags.contains(LightFlags::AT_NODE) {
            let node = if state.node_index == index!(input) {
                INPUT_NODE_NAME.to_string()
            } else {
                anim_def.node_from_index(state.node_index, read.prev + 56)?
            };
            Some(Translate::AtNode(AtNode {
                name: node,
                pos: state.translate,
            }))
        } else if flags.contains(LightFlags::TRANSLATE_ABS) {
            assert_that!(
                "light state at node index",
                state.node_index == index!(0),
                read.prev + 56
            )?;
            Some(Translate::Absolute(state.translate))
        } else {
            assert_that!(
                "light state at node index",
                state.node_index == index!(0),
                read.prev + 56
            )?;
            assert_that!(
                "light state translation",
                state.translate == Vec3::DEFAULT,
                read.prev + 60
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
                read.prev + 72
            )?;
            None
        };

        let range = if flags.contains(LightFlags::RANGE) {
            assert_that!(
                "light state range near",
                state.range.min >= 0.0,
                read.prev + 84
            )?;
            assert_that!(
                "light state range far",
                state.range.max >= state.range.min,
                read.prev + 88
            )?;
            Some(state.range)
        } else {
            assert_that!(
                "light state range",
                state.range == Range::DEFAULT,
                read.prev + 84
            )?;
            None
        };

        let color = if flags.contains(LightFlags::COLOR) {
            assert_color!("light state color", state.color, read.prev + 92)?;
            Some(state.color)
        } else {
            assert_that!(
                "light state color",
                state.color == Color::BLACK,
                read.prev + 92
            )?;
            None
        };

        let ambient = if flags.contains(LightFlags::AMBIENT) {
            assert_that!("light state ambient", 0.0 <= state.ambient <= 1.0, read.prev + 104)?;
            Some(state.ambient)
        } else {
            assert_that!("light state ambient", state.ambient == 0.0, read.prev + 104)?;
            None
        };

        let diffuse = if flags.contains(LightFlags::DIFFUSE) {
            assert_that!("light state diffuse", 0.0 <= state.diffuse <= 1.0, read.prev + 108)?;
            Some(state.diffuse)
        } else {
            assert_that!("light state diffuse", state.diffuse == 0.0, read.prev + 108)?;
            None
        };

        Ok(Self {
            name,
            active_state,
            type_: light_type,
            directional,
            saturated,
            subdivide: None,
            lightmap: None,
            static_: None,
            bicolored: None,
            translate,
            orientation,
            range,
            color,
            ambient_color: None,
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

        let light_state = LightStateRcC {
            light_name,
            light_index,
            flags: flags.maybe(),
            active_state: self.active_state.into(),
            ty_: self.type_.maybe(),
            directional: self.directional.unwrap_or(false).into(),
            saturated: self.saturated.unwrap_or(false).into(),
            node_index,
            translate,
            orientation: self.orientation.unwrap_or(Vec3::DEFAULT),
            range: self.range.unwrap_or(Range::DEFAULT),
            color: self.color.unwrap_or(Color::BLACK),
            ambient: self.ambient.unwrap_or(0.0),
            diffuse: self.diffuse.unwrap_or(0.0),
        };
        write.write_struct(&light_state)?;
        Ok(())
    }
}
