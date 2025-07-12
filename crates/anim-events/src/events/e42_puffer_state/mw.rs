use super::{
    EventMw, PufferStateColors, PufferStateCommon, PufferStateGrowths, PufferStateTextures, read,
    write,
};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::PufferState;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStateMwC {
    common: PufferStateCommon,     // 000
    textures: PufferStateTextures, // 192
    colors: PufferStateColors,     // 404
    growths: PufferStateGrowths,   // 528
}
impl_as_bytes!(PufferStateMwC, 580);

impl EventMw for PufferState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(PufferStateMwC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "puffer state size",
            size == PufferStateMwC::SIZE,
            read.offset
        )?;
        let PufferStateMwC {
            common,
            textures,
            colors,
            growths,
        } = read.read_struct()?;

        let (flags, mut state) = read::assert_common(common, anim_def, read.prev + 0)?;
        state.textures = read::assert_textures(textures, flags, read.prev + 192)?;
        state.colors = read::assert_colors(colors, flags, read.prev + 404)?;
        state.growth_factors = read::assert_growths(growths, flags, read.prev + 528)?;

        Ok(state)
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let common = write::make_common(self, anim_def)?;
        let textures = write::make_textures(&self.textures)?;
        let colors = write::make_colors(&self.colors)?;
        let growths = write::make_growths(&self.growth_factors)?;

        let puffer_state = PufferStateMwC {
            common,
            textures,
            colors,
            growths,
        };
        write.write_struct(&puffer_state)?;
        Ok(())
    }
}
