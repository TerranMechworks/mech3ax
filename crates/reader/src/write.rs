use super::{FLOAT, INT, LIST, STRING};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Result};
use serde_json::{Number, Value};
use std::convert::TryInto as _;
use std::io::Write;

pub fn write_reader(write: &mut CountingWriter<impl Write>, value: &Value) -> Result<()> {
    write_value(write, value)
}

fn write_value(write: &mut CountingWriter<impl Write>, value: &Value) -> Result<()> {
    match value {
        Value::String(value) => write_string(write, value),
        Value::Null => write_empty_list(write),
        Value::Array(value) => write_list(write, value),
        Value::Number(value) => write_number(write, value),
        Value::Bool(value) => Err(assert_with_msg!(
            "Readers do not support booleans: {}",
            value
        )),
        Value::Object(value) => Err(assert_with_msg!(
            "Readers do not support objects: {:#?}",
            value
        )),
    }
}

fn write_string(write: &mut CountingWriter<impl Write>, value: &str) -> Result<()> {
    write.write_u32(STRING)?;
    write.write_string(value)?;
    Ok(())
}

fn write_empty_list(write: &mut CountingWriter<impl Write>) -> Result<()> {
    const COUNT: u32 = 1;
    write.write_u32(LIST)?;
    write.write_u32(COUNT)?;
    Ok(())
}

fn write_list(write: &mut CountingWriter<impl Write>, value: &[Value]) -> Result<()> {
    let count = assert_len!(u32, value.len() + 1, "reader list")?;
    write.write_u32(LIST)?;
    write.write_u32(count)?;
    for item in value.iter() {
        write_value(write, item)?;
    }
    Ok(())
}

fn write_number(write: &mut CountingWriter<impl Write>, value: &Number) -> Result<()> {
    // Unpacking a `Number` is difficult, due to the `arbitrary_precision`
    // feature. The current serde behaviour is:
    // * `as_u64()` only matches `N::PosInt(u64)`
    // * `as_i64()` matches `N::PosInt(u64)` if `n <= i64::MAX` or
    //   `N::NegInt(i64)`
    // * `as_f64()` matches `N::PosInt(u64)` via casting `as f64`,
    //   `N::NegInt(i64) via casting `as f64`, or `N::Float(f64)`
    // * `as_f32()` is like `as_f64()`, but simply casts to `f32`
    // Therefore, it's important to try `as_u64()` first, then `as_i64()`, and
    // finally `as_f64()`.
    if let Some(pos_int) = value.as_u64() {
        let int = pos_int.try_into().map_err(|_| {
            assert_with_msg!(
                "Reader integer must be >= {}, <= {}, but was {}",
                i32::MIN,
                i32::MAX,
                pos_int
            )
        })?;
        write.write_u32(INT)?;
        write.write_i32(int)?;
        Ok(())
    } else if let Some(neg_int) = value.as_i64() {
        let int = neg_int.try_into().map_err(|_| {
            assert_with_msg!(
                "Reader integer must be >= {}, <= {}, but was {}",
                i32::MIN,
                i32::MAX,
                neg_int
            )
        })?;
        write.write_u32(INT)?;
        write.write_i32(int)?;
        Ok(())
    } else if let Some(double) = value.as_f64() {
        // this causes precision-loss, but might be necessary?
        let float = double as f32;
        write.write_u32(FLOAT)?;
        write.write_f32(float)?;
        Ok(())
    } else {
        unreachable!("Unknown number representation: {:?}", value);
    }
}
