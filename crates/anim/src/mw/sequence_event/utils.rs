use mech3ax_api_types::Color;
use mech3ax_common::{assert_that, Result};

pub fn assert_color(name: &str, color: &Color, offset: usize) -> Result<()> {
    assert_that!(format!("{} color red", name), 0.0 <= color.r <= 1.0, offset + 0)?;
    assert_that!(format!("{} color green", name), 0.0 <= color.g <= 1.0, offset + 4)?;
    assert_that!(format!("{} color blue", name), 0.0 <= color.b <= 1.0, offset + 8)?;
    Ok(())
}
