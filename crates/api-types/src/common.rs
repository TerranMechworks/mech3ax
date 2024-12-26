use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_types::impl_as_bytes;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct Range {
    pub min: f32,
    pub max: f32,
}
impl_as_bytes!(Range, 8);

impl Range {
    pub const DEFAULT: Self = Self { min: 0.0, max: 0.0 };
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
    Default,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl_as_bytes!(Vec3, 12);

impl Vec3 {
    pub const DEFAULT: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl_as_bytes!(Color, 12);

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE_FULL: Self = Self {
        r: 255.0,
        g: 255.0,
        b: 255.0,
    };
    pub const WHITE_NORM: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
    Default,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl_as_bytes!(Quaternion, 16);

impl Quaternion {
    pub const DEFAULT: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
    Default,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
    pub g: f32,
    pub h: f32,
    pub i: f32,
}
impl_as_bytes!(Matrix, 36);

impl Matrix {
    pub const EMPTY: Self = Self {
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
        g: 0.0,
        h: 0.0,
        i: 0.0,
    };
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 1.0,
        f: 0.0,
        g: 0.0,
        h: 0.0,
        i: 1.0,
    };
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
    Default,
)]
#[dotnet(val_struct)]
#[repr(C)]
/// A cut-down affine matrix, without the 4th row.
pub struct AffineMatrix {
    pub r00: f32,
    pub r01: f32,
    pub r02: f32,
    pub r10: f32,
    pub r11: f32,
    pub r12: f32,
    pub r20: f32,
    pub r21: f32,
    pub r22: f32,
    pub r30: f32,
    pub r31: f32,
    pub r32: f32,
}
impl_as_bytes!(AffineMatrix, 48);

impl AffineMatrix {
    pub const ZERO: Self = Self {
        r00: 0.0,
        r01: 0.0,
        r02: 0.0,
        r10: 0.0,
        r11: 0.0,
        r12: 0.0,
        r20: 0.0,
        r21: 0.0,
        r22: 0.0,
        r30: 0.0,
        r31: 0.0,
        r32: 0.0,
    };
}
