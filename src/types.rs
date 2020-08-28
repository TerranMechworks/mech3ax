use crate::static_assert_size;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec2(f32, f32);
static_assert_size!(Vec2, 8);

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec3(f32, f32, f32);
static_assert_size!(Vec3, 12);

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec4(f32, f32, f32, f32);
static_assert_size!(Vec4, 16);
