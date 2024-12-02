use super::{
    read, write, EventPm, PufferStateColors, PufferStateCommon, PufferStateFlags,
    PufferStateGrowths, PufferStateTextures,
};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::PufferState;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStatePmC {
    common: PufferStateCommon,     // 000
    number: u32,                   // 184
    textures: PufferStateTextures, // 188
    colors: PufferStateColors,     // 408
    growths: PufferStateGrowths,   // 532
}
impl_as_bytes!(PufferStatePmC, 584);

impl EventPm for PufferState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(PufferStatePmC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "puffer state size",
            size == PufferStatePmC::SIZE,
            read.offset
        )?;
        let PufferStatePmC {
            common,
            number,
            textures,
            colors,
            growths,
        } = read.read_struct()?;

        let (flags, mut state) = read::assert_common(common, anim_def, read.prev + 0)?;

        state.number = if flags.contains(PufferStateFlags::NUMBER) {
            assert_that!("puffer state number", number > 0, read.prev + 184)?;
            Some(number)
        } else {
            assert_that!("puffer state number", number == 0, read.prev + 184)?;
            None
        };
        state.textures = read::assert_textures(textures, flags, read.prev + 196)?;
        state.colors = read::assert_colors(colors, flags, read.prev + 408)?;
        state.growth_factors = read::assert_growths(growths, flags, read.prev + 532)?;

        Ok(state)
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let common = write::make_common(self, anim_def)?;
        let number = self.number.unwrap_or(0);
        let textures = write::make_textures(&self.textures)?;
        let colors = write::make_colors(&self.colors)?;
        let growths = write::make_growths(&self.growth_factors)?;

        let puffer_state = PufferStatePmC {
            common,
            number,
            textures,
            colors,
            growths,
        };
        write.write_struct(&puffer_state)?;
        Ok(())
    }
}
