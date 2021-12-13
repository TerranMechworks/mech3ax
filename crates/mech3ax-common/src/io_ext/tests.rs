use super::*;
use crate::string::str_to_c_padded;
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
    let mut input = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(3735928559, input.read_u32().unwrap());

    let mut output = Cursor::new(vec![]);
    output.write_u32(3735928559).unwrap();
    assert_eq!(expected, output.read_all());
}

#[test]
fn i32_roundtrip() {
    let expected = vec![0xEF, 0xBE, 0xAD, 0xDE];
    let mut input = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(-559038737, input.read_i32().unwrap());

    let mut output = Cursor::new(vec![]);
    output.write_i32(-559038737).unwrap();
    assert_eq!(expected, output.read_all());
}

#[test]
fn f32_roundtrip() {
    let expected = -1.0f32;
    let mut cursor = Cursor::new(vec![]);
    cursor.write_f32(expected).unwrap();
    cursor.set_position(0);
    let mut reader = CountingReader::new(cursor);
    let actual = reader.read_f32().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn u16_roundtrip() {
    let expected = vec![0xEF, 0xBE];
    let mut input = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(48879, input.read_u16().unwrap());

    let mut output = Cursor::new(vec![]);
    output.write_u16(48879).unwrap();
    assert_eq!(expected, output.read_all());
}

#[test]
fn i16_roundtrip() {
    let expected = vec![0xEF, 0xBE];
    let mut input = CountingReader::new(Cursor::new(expected.clone()));
    assert_eq!(-16657, input.read_i16().unwrap());

    let mut output = Cursor::new(vec![]);
    output.write_i16(-16657).unwrap();
    assert_eq!(expected, output.read_all());
}

#[derive(Debug, PartialEq)]
#[repr(C)]
struct TestStruct {
    name: [u8; 32],
    int: u32,
}
crate::static_assert_size!(TestStruct, 36);

#[test]
fn struct_roundtrip() {
    let mut name = [0u8; 32];
    str_to_c_padded("Hello World", &mut name);
    let expected = TestStruct {
        name,
        int: 3735928559,
    };

    let mut cursor = Cursor::new(vec![]);
    cursor.write_struct(&expected).unwrap();
    assert_eq!(std::mem::size_of::<TestStruct>() as u64, cursor.position());

    cursor.set_position(0);
    let mut reader = CountingReader::new(cursor);
    let actual: TestStruct = reader.read_struct().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn string_roundtrip() {
    let expected = "Hello World".to_owned();
    let mut cursor = Cursor::new(vec![]);
    cursor.write_string(&expected).unwrap();
    cursor.set_position(0);
    let mut reader = CountingReader::new(cursor);
    let actual = reader.read_string().unwrap();
    assert_eq!(expected, actual);
    assert_eq!(reader.offset as usize, expected.len() + 4);
}
