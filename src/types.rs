use crate::static_assert_size;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec2(pub f32, pub f32);
static_assert_size!(Vec2, 8);

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec3(pub f32, pub f32, pub f32);
static_assert_size!(Vec3, 12);

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec4(pub f32, pub f32, pub f32, pub f32);
static_assert_size!(Vec4, 16);

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
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
