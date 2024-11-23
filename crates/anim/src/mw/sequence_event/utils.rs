use mech3ax_api_types::Color;
use mech3ax_common::{assert_that, Result};

pub(crate) fn _assert_color(
    name_r: &str,
    name_g: &str,
    name_b: &str,
    color: &Color,
    offset: usize,
) -> Result<()> {
    assert_that!(name_r, 0.0 <= color.r <= 1.0, offset + 0)?;
    assert_that!(name_g, 0.0 <= color.g <= 1.0, offset + 4)?;
    assert_that!(name_b, 0.0 <= color.b <= 1.0, offset + 8)?;
    Ok(())
}

macro_rules! assert_color {
    ($name:literal, $color:expr, $offset:expr) => {
        crate::mw::sequence_event::utils::_assert_color(
            concat!($name, " color r"),
            concat!($name, " color g"),
            concat!($name, " color g"),
            &$color,
            $offset,
        )
    };
}
pub(crate) use assert_color;
