use crate::static_assert_size;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct Vec2(pub f32, pub f32);
static_assert_size!(Vec2, 8);

impl Vec2 {
    pub const EMPTY: Vec2 = Vec2(0.0, 0.0);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct Vec3(pub f32, pub f32, pub f32);
static_assert_size!(Vec3, 12);

impl Vec3 {
    pub const EMPTY: Vec3 = Vec3(0.0, 0.0, 0.0);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct Vec4(pub f32, pub f32, pub f32, pub f32);
static_assert_size!(Vec4, 16);

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
