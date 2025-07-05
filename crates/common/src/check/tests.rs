use crate::chk;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Offsets};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct Bar {
    x: u32, // 00 / 04
    y: u32, // 04 / 08
}
impl_as_bytes!(Bar, 8);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct Foo {
    a: u32, // 00
    b: Bar, // 04
    c: u32, // 12
}
impl_as_bytes!(Foo, 16);

#[test]
fn level1() {
    let foo = Foo {
        a: 1,
        b: Bar { x: 41, y: 42 },
        c: 3,
    };

    let name = chk!(__name foo.a);
    assert_eq!(name, "foo.a");
    let offset = chk!(__offset foo.a, 0);
    assert_eq!(offset, 0);
    let offset = chk!(__offset foo.a, 1000);
    assert_eq!(offset, 1000);

    let name = chk!(__name foo.b);
    assert_eq!(name, "foo.b");
    let offset = chk!(__offset foo.b, 0);
    assert_eq!(offset, 4);
    let offset = chk!(__offset foo.b, 1000);
    assert_eq!(offset, 1004);

    let name = chk!(__name foo.c);
    assert_eq!(name, "foo.c");
    let offset = chk!(__offset foo.c, 0);
    assert_eq!(offset, 12);
    let offset = chk!(__offset foo.c, 1000);
    assert_eq!(offset, 1012);
}

#[test]
fn level2() {
    let foo = Foo {
        a: 1,
        b: Bar { x: 41, y: 42 },
        c: 3,
    };

    let name = chk!(__name foo.b.x);
    assert_eq!(name, "foo.b.x");
    let offset = chk!(__offset foo.b.x, 0);
    assert_eq!(offset, 4);
    let offset = chk!(__offset foo.b.x, 1000);
    assert_eq!(offset, 1004);

    let name = chk!(__name foo.b.y);
    assert_eq!(name, "foo.b.y");
    let offset = chk!(__offset foo.b.y, 0);
    assert_eq!(offset, 8);
    let offset = chk!(__offset foo.b.y, 1000);
    assert_eq!(offset, 1008);
}
