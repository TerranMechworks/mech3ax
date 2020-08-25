use crate::assert::{assert_utf8, AssertionError};
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::Result;
use std::convert::TryInto;
use std::io::{Read, Write};

use serde_json::{Number, Value};

fn read_value<R>(read: &mut R, offset: &mut usize) -> Result<Value>
where
    R: Read,
{
    let value_type = read.read_u32()?;
    *offset += 4;

    match value_type {
        1 => {
            let value = read.read_i32()?;
            *offset += 4;
            Ok(Value::Number(Number::from(value)))
        }
        2 => {
            let value = read.read_f32()?;
            *offset += 4;
            Ok(Value::Number(Number::from_f64(value as f64).unwrap()))
        }
        3 => {
            let count = read.read_u32()? as usize;
            *offset += 4;
            let mut buf = vec![0u8; count];
            read.read_exact(&mut buf)?;
            let value = assert_utf8("value", *offset, || std::str::from_utf8(&buf))?;
            *offset += count;
            Ok(Value::String(value.to_owned()))
        }
        4 => {
            // count is one bigger, because the engine stores the count as an
            // integer node as the first item of the list
            let count = read.read_u32()? - 1;
            *offset += 4;

            if count == 0 {
                Ok(Value::Null)
            } else {
                let value = (0..count)
                    .into_iter()
                    .map(|_| read_value(read, offset))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Value::Array(value))
            }
        }
        _ => {
            let msg = format!(
                "Expected valid value type, but was {} (at {})",
                value_type, *offset
            );
            Err(AssertionError(msg))?
        }
    }
}

pub fn read_reader<R>(read: &mut R) -> Result<Value>
where
    R: Read,
{
    let mut offset = 0;
    read_value(read, &mut offset)
}

fn invalid_number(num: Number) -> AssertionError {
    AssertionError(format!("Expected valid number, but was {}", num))
}

fn write_value<W>(write: &mut W, value: Value) -> Result<()>
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
                Err(invalid_number(num))?
            }
        }
        Value::String(str) => {
            write.write_u32(3)?;
            let buf = str.into_bytes();
            let count = buf.len() as u32;
            write.write_u32(count)?;
            write.write_all(&buf)?;
        }
        Value::Null => {
            write.write_u32(4)?;
            write.write_u32(1)?; // count
        }
        Value::Array(list) => {
            write.write_u32(4)?;
            let count = list.len() as u32 + 1;
            write.write_u32(count)?;

            for item in list.into_iter() {
                write_value(write, item)?;
            }
        }
        _ => {
            let msg = format!("Expected valid value type, but was {}", value);
            Err(AssertionError(msg))?
        }
    };
    Ok(())
}

pub fn write_reader<W>(write: &mut W, value: Value) -> Result<()>
where
    W: Write,
{
    write_value(write, value)
}
