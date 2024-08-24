use super::{FLOAT, INT, LIST, STRING};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_with_msg, Result};
use serde_json::{Number, Value};
use std::io::Read;

pub fn read_reader(read: &mut CountingReader<impl Read>) -> Result<Value> {
    let value = read_value(read);
    if value.is_ok() {
        read.assert_end()?;
    }
    value
}

fn read_value(read: &mut CountingReader<impl Read>) -> Result<Value> {
    match read.read_u32()? {
        INT => read_int(read),
        FLOAT => read_float(read),
        STRING => read_string(read),
        LIST => read_list(read),
        value_type => Err(assert_with_msg!(
            "Expected value type to be {}, {}, {} or {}, but was {} (at {})",
            INT,
            FLOAT,
            STRING,
            LIST,
            value_type,
            read.prev
        )),
    }
}

fn read_int(read: &mut CountingReader<impl Read>) -> Result<Value> {
    let int = read.read_i32()?;
    Ok(Value::Number(Number::from(int)))
}

fn read_float(read: &mut CountingReader<impl Read>) -> Result<Value> {
    let float = read.read_f32()?;
    let double = f64::from(float);
    let number = Number::from_f64(double).ok_or_else(|| {
        assert_with_msg!(
            "Expected finite float, but was {:?} (value: {}, at {})",
            float.classify(),
            float,
            read.prev,
        )
    })?;
    Ok(Value::Number(number))
}

fn read_string(read: &mut CountingReader<impl Read>) -> Result<Value> {
    Ok(Value::String(read.read_string()?))
}

fn read_list(read: &mut CountingReader<impl Read>) -> Result<Value> {
    let count = read.read_u32()?;
    // count is one bigger, maybe the engine stored the count as the first item?
    let len = count
        .checked_sub(1)
        .ok_or_else(|| assert_with_msg!("Expected list count > 0, but was (at {})", read.prev))?;
    if len == 0 {
        return Ok(Value::Null);
    }
    let value = (0..len)
        .map(|_| read_value(read))
        .collect::<Result<Vec<_>>>()?;
    Ok(Value::Array(value))
}
