use crate::assert::AssertionError;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::Result;
use std::convert::TryInto;
use std::io::{Read, Write};

use serde_json::{Number, Value};

fn read_value<R>(read: &mut CountingReader<R>) -> Result<Value>
where
    R: Read,
{
    match read.read_u32()? {
        1 => Ok(Value::Number(Number::from(read.read_i32()?))),
        2 => Ok(Value::Number(
            Number::from_f64(read.read_f32()? as f64).unwrap(),
        )),
        3 => Ok(Value::String(read.read_string()?)),
        4 => {
            // count is one bigger, because the engine stores the count as an
            // integer node as the first item of the list
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
        value_type => {
            let msg = format!(
                "Expected valid value type, but was {} (at {})",
                value_type, read.prev
            );
            Err(AssertionError(msg).into())
        }
    }
}

pub fn read_reader<R>(read: &mut CountingReader<R>) -> Result<Value>
where
    R: Read,
{
    let value = read_value(read);
    read.assert_end()?;
    value
}

fn invalid_number(num: &Number) -> AssertionError {
    AssertionError(format!("Expected valid number, but was {}", num))
}

fn write_value<W>(write: &mut W, value: &Value) -> Result<()>
where
    W: Write,
{
    match value {
        Value::Number(num) => {
            if let Some(int) = num.as_i64() {
                let int = int.try_into().map_err(|_| invalid_number(num))?;
                write.write_u32(1)?;
                write.write_i32(int)?;
            } else if let Some(float) = num.as_f64() {
                write.write_u32(2)?;
                write.write_f32(float as f32)?;
            } else {
                return Err(invalid_number(num).into());
            }
        }
        Value::String(string) => {
            write.write_u32(3)?;
            write.write_string(&string)?;
        }
        Value::Null => {
            write.write_u32(4)?;
            write.write_u32(1)?; // count
        }
        Value::Array(list) => {
            write.write_u32(4)?;
            let count = list.len() as u32 + 1;
            write.write_u32(count)?;

            for item in list.iter() {
                write_value(write, item)?;
            }
        }
        _ => {
            let msg = format!("Expected valid value type, but was {}", value);
            return Err(AssertionError(msg).into());
        }
    };
    Ok(())
}

pub fn write_reader<W>(write: &mut W, value: &Value) -> Result<()>
where
    W: Write,
{
    write_value(write, value)
}
