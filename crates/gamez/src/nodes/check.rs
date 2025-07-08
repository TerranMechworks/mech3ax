use mech3ax_api_types::{Count, Index};
use mech3ax_types::Ptr;

type Result<T> = std::result::Result<T, String>;

pub(crate) fn ap(value: i32) -> Result<u8> {
    u8::try_from(value).map_err(|_e| format!("expected {} in 0..={}", value, u8::MAX))
}

pub(crate) fn model_index(value: i32) -> Result<Option<Index>> {
    if value == -1 {
        return Ok(None);
    }
    Index::check_i32(value).map(Some)
}

pub(crate) fn node_count(value: i32) -> Result<Count> {
    Count::check_i32(value)
}

pub(crate) fn node_index(value: i32) -> Result<Option<Index>> {
    if value == -1 {
        return Ok(None);
    }
    Index::check_i32(value).map(Some)
}

pub(crate) fn ptr(value: Ptr, count: Count) -> Result<Ptr> {
    if count == 0 {
        if value == Ptr::NULL {
            Ok(value)
        } else {
            Err(format!("expected {:?} == NULL", value))
        }
    } else {
        if value != Ptr::NULL {
            Ok(value)
        } else {
            Err(format!("expected {:?} != NULL", value))
        }
    }
}

pub(crate) fn color(value: f32) -> Result<f32> {
    if value < 0.0 || value > 1.0 {
        Err(format!("expected 0.0 <= {} <= 1.0", value))
    } else {
        Ok(value)
    }
}
