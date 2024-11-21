use super::*;
use crate::string::str_to_c_padded;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Ascii};
use std::io::Cursor;

trait ReadAll {
    fn read_all(&mut self) -> Vec<u8>;
}

impl ReadAll for Cursor<Vec<u8>> {
    fn read_all(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.set_position(0);
        self.read_to_end(&mut buf).unwrap();
        self.set_position(0);
        buf
    }
}

#[test]
fn u32_roundtrip() {
    let expected = vec![0xEF, 0xBE, 0xAD, 0xDE];
    let mut reader = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(3735928559, reader.read_u32().unwrap());

    let mut writer = CountingWriter::new(Cursor::new(vec![]), 0);
    writer.write_u32(3735928559).unwrap();

    let mut cursor = writer.into_inner();
    assert_eq!(expected, cursor.read_all());
}

#[test]
fn i32_roundtrip() {
    let expected = vec![0xEF, 0xBE, 0xAD, 0xDE];
    let mut reader = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(-559038737, reader.read_i32().unwrap());

    let mut writer = CountingWriter::new(Cursor::new(vec![]), 0);
    writer.write_i32(-559038737).unwrap();

    let mut cursor = writer.into_inner();
    assert_eq!(expected, cursor.read_all());
}

#[test]
fn f32_roundtrip() {
    let expected = -1.0f32;
    let mut writer = CountingWriter::new(Cursor::new(vec![]), 0);
    writer.write_f32(expected).unwrap();

    let mut cursor = writer.into_inner();
    cursor.set_position(0);
    let mut reader = CountingReader::new(cursor);
    let actual = reader.read_f32().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn u16_roundtrip() {
    let expected = vec![0xEF, 0xBE];
    let mut reader = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(48879, reader.read_u16().unwrap());

    let mut writer = CountingWriter::new(Cursor::new(vec![]), 0);
    writer.write_u16(48879).unwrap();

    let mut cursor = writer.into_inner();
    assert_eq!(expected, cursor.read_all());
}

#[test]
fn i16_roundtrip() {
    let expected = vec![0xEF, 0xBE];
    let mut reader = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(-16657, reader.read_i16().unwrap());

    let mut writer = CountingWriter::new(Cursor::new(vec![]), 0);
    writer.write_i16(-16657).unwrap();

    let mut cursor = writer.into_inner();
    assert_eq!(expected, cursor.read_all());
}

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TestStruct {
    name: Ascii<32>,
    int: u32,
}
impl_as_bytes!(TestStruct, 36);

#[test]
fn struct_roundtrip() {
    let mut name: Ascii<32> = Ascii::zero();
    str_to_c_padded("Hello World", &mut name);
    let expected = TestStruct {
        name,
        int: 3735928559,
    };

    let mut cursor = CountingWriter::new(Cursor::new(vec![]), 0);
    cursor.write_struct(&expected).unwrap();
    assert_eq!(
        std::mem::size_of::<TestStruct>() as u64,
        cursor.get_mut().position()
    );

    let mut cursor = cursor.into_inner();
    cursor.set_position(0);
    let mut reader = CountingReader::new(cursor);
    let actual: TestStruct = reader.read_struct().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn string_roundtrip() {
    let expected = "Hello World".to_owned();
    let mut writer = CountingWriter::new(Cursor::new(vec![]), 0);
    writer.write_string(&expected).unwrap();

    let mut cursor = writer.into_inner();
    cursor.set_position(0);
    let mut reader = CountingReader::new(cursor);
    let actual = reader.read_string().unwrap();
    assert_eq!(expected, actual);
    assert_eq!(reader.offset as usize, expected.len() + 4);
}
