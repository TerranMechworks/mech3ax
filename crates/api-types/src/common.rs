use crate::api;
use mech3ax_types::impl_as_bytes;

api! {
    #[repr(C)]
    struct Range : Val {
        min: f32,
        max: f32,
    }
}
impl_as_bytes!(Range, 8);

impl Range {
    pub const DEFAULT: Self = Self { min: 0.0, max: 0.0 };
}

api! {
    #[repr(C)]
    struct Vec3 : Val {
        x: f32,
        y: f32,
        z: f32,
    }
}
impl_as_bytes!(Vec3, 12);

impl Default for Vec3 {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Vec3 {
    pub const DEFAULT: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

api! {
    #[repr(C)]
    struct Color : Val {
        r: f32,
        g: f32,
        b: f32,
    }
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

api! {
    #[repr(C)]
    struct Quaternion : Val {
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    }
}
impl_as_bytes!(Quaternion, 16);

impl Default for Quaternion {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Quaternion {
    pub const DEFAULT: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
}

api! {
    #[repr(C)]
    struct Matrix {
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        e: f32,
        f: f32,
        g: f32,
        h: f32,
        i: f32,
    }
}
impl_as_bytes!(Matrix, 36);

impl Default for Matrix {
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}

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

api! {
    /// A cut-down affine matrix, without the 4th row.
    #[repr(C)]
    struct AffineMatrix {
        r00: f32,
        r01: f32,
        r02: f32,
        r10: f32,
        r11: f32,
        r12: f32,
        r20: f32,
        r21: f32,
        r22: f32,
        r30: f32,
        r31: f32,
        r32: f32,
    }
}
impl_as_bytes!(AffineMatrix, 48);

impl Default for AffineMatrix {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl AffineMatrix {
    pub const DEFAULT: Self = Self {
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

    pub const IDENTITY: Self = Self {
        r00: 1.0,
        r01: 0.0,
        r02: 0.0,
        r10: 0.0,
        r11: 1.0,
        r12: 0.0,
        r20: 0.0,
        r21: 0.0,
        r22: 1.0,
        r30: 0.0,
        r31: 0.0,
        r32: 0.0,
    };

    #[inline]
    pub fn translate(&self) -> Vec3 {
        Vec3 {
            x: self.r30,
            y: self.r31,
            z: self.r32,
        }
    }
}
