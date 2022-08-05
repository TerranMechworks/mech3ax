use mech3ax_api_types::Vec3;
use mech3ax_common::{assert_that, Result};

pub fn assert_color(name: &str, color: &Vec3, offset: u32) -> Result<()> {
    assert_that!(format!("{} color red", name), 0.0 <= color.0 <= 1.0, offset + 0)?;
    assert_that!(format!("{} color green", name), 0.0 <= color.1 <= 1.0, offset + 4)?;
    assert_that!(format!("{} color blue", name), 0.0 <= color.2 <= 1.0, offset + 8)?;
    Ok(())
}
