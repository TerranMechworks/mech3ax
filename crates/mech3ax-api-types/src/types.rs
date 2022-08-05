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
static_assert_size!(Matrix, 36);

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
