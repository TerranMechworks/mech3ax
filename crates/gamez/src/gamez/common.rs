use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::{Count, Count32};
use mech3ax_common::{chk, err, Result};
use mech3ax_types::{impl_as_bytes, Offsets};
pub(crate) const SIGNATURE: u32 = 0x02971222;

pub(crate) const VERSION_RC: u32 = 15;
pub(crate) const VERSION_MW: u32 = 27;
pub(crate) const VERSION_PM: u32 = 41;
#[expect(dead_code)]
pub(crate) const VERSION_CS: u32 = 42;

pub(crate) fn texture_count(value: Count32) -> Result<Count, String> {
    let v: i32 = value.value;
    if (0..4096).contains(&v) {
        value.check()
    } else {
        Err(format!("expected {} in 0..4096", v))
    }
}

// we'll never know why???
pub(crate) const NODE_INDEX_INVALID: i32 = 0x00FFFFFF;

pub(crate) const NODE_INDEX_TOP_MASK: u32 = 0xFF000000;
pub(crate) const NODE_INDEX_BOT_MASK: u32 = 0x00FFFFFF;
pub(crate) const NODE_INDEX_TOP: u32 = 0x02000000;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
pub(crate) struct ModelArrayC {
    array_size: Count32, // 00
    count: Count32,      // 04
    index_free: i32,     // 08
}
impl_as_bytes!(ModelArrayC, 12);

pub(crate) fn assert_model_array(
    model_array: &ModelArrayC,
    offset: usize,
) -> Result<(Count, Count)> {
    let model_array_size = chk!(offset, ?model_array.array_size)?;
    let model_count = chk!(offset, ?model_array.count)?;
    // technically, we could handle count == array_size, but then index_free
    // would have to be -1
    chk!(offset, model_count < model_array_size)?;

    let index_free = model_count.to_i32();
    chk!(offset, model_array.index_free == index_free)?;

    Ok((model_count, model_array_size))
}

pub(crate) fn make_model_array(count: Count, array_size: Count) -> Result<ModelArrayC> {
    // technically, we could handle count == array_size, but then index_free
    // would have to be -1
    if count >= array_size {
        return Err(err!(
            "Too many GameZ models: expected {} < {}",
            count,
            array_size
        ));
    }
    let index_free = count.to_i32();

    Ok(ModelArrayC {
        array_size: array_size.maybe(),
        count: count.maybe(),
        index_free,
    })
}
