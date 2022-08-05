use crate::static_assert_size;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct Range {
    pub min: f32,
    pub max: f32,
}
static_assert_size!(Range, 8);

impl Range {
    pub const DEFAULT: Self = Self { min: 0.0, max: 0.0 };
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct Vec3(pub f32, pub f32, pub f32);
static_assert_size!(Vec3, 12);

impl Vec3 {
    pub const DEFAULT: Vec3 = Vec3(0.0, 0.0, 0.0);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
static_assert_size!(Color, 12);

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

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
static_assert_size!(Quaternion, 16);

impl Quaternion {
    pub const DEFAULT: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
#[repr(C)]
pub struct Matrix(
    pub f32,
    pub f32,
    pub f32,
    pub f32,
    pub f32,
    pub f32,
    pub f32,
    pub f32,
    pub f32,
);
static_assert_size!(Matrix, 36);

impl Matrix {
    pub const EMPTY: Matrix = Matrix(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const IDENTITY: Matrix = Matrix(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
}
