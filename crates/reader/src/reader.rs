use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_with_msg, Result};
use std::convert::TryInto;
use std::io::{Read, Write};

use serde_json::{Number, Value};

const INT: u32 = 1;
const FLOAT: u32 = 2;
const STRING: u32 = 3;
const LIST: u32 = 4;

fn read_value(read: &mut CountingReader<impl Read>) -> Result<Value> {
    match read.read_u32()? {
        INT => Ok(Value::Number(Number::from(read.read_i32()?))),
        FLOAT => Ok(Value::Number(
            Number::from_f64(read.read_f32()? as f64).unwrap(),
        )),
        STRING => Ok(Value::String(read.read_string()?)),
        LIST => {
            // count is one bigger, maybe the engine stored the count as the first item?
            let count = read.read_u32()? - 1;
            if count == 0 {
                Ok(Value::Null)
            } else {
                let value = (0..count)
                    .map(|_| read_value(read))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Value::Array(value))
            }
        }
        value_type => Err(assert_with_msg!(
            "Expected valid value type, but was {} (at {})",
            value_type,
            read.prev
        )),
    }
}

pub fn read_reader(read: &mut CountingReader<impl Read>) -> Result<Value> {
    let value = read_value(read);
    if value.is_ok() {
        read.assert_end()?;
    }
    value
}

fn invalid_number(num: &Number) -> mech3ax_common::Error {
    assert_with_msg!("Expected valid number, but was {}", num)
}

fn write_value(write: &mut CountingWriter<impl Write>, value: &Value) -> Result<()> {
    match value {
        Value::Number(num) => {
            if let Some(int) = num.as_i64() {
                let int = int.try_into().map_err(|_| invalid_number(num))?;
                write.write_u32(INT)?;
                write.write_i32(int)?;
            } else if let Some(float) = num.as_f64() {
                write.write_u32(FLOAT)?;
                write.write_f32(float as f32)?;
            } else {
                return Err(invalid_number(num));
            }
        }
        Value::String(string) => {
            write.write_u32(STRING)?;
            write.write_string(string)?;
        }
        Value::Null => {
            write.write_u32(LIST)?;
            write.write_u32(1)?; // count
        }
        Value::Array(list) => {
            let length = list.len() + 1;
            let count = length.try_into().map_err(|_| {
                assert_with_msg!(
                    "Expected list to have {} items or fewer, but was {}",
                    u32::MAX,
                    length
                )
            })?;
            write.write_u32(LIST)?;
            write.write_u32(count)?;

            for item in list.iter() {
                write_value(write, item)?;
            }
        }
        _ => {
            return Err(assert_with_msg!(
                "Expected valid value type, but was {}",
                value
            ))
        }
    };
    Ok(())
}

pub fn write_reader(write: &mut CountingWriter<impl Write>, value: &Value) -> Result<()> {
    write_value(write, value)
}
