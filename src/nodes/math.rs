use crate::types::{Matrix, Vec3};

pub const IDENTITY_MATRIX: Matrix = Matrix(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
pub const PI: f32 = std::f64::consts::PI as f32;

pub fn euler_to_matrix(rotation: &Vec3) -> Matrix {
    let x = -rotation.0;
    let y = -rotation.1;
    let z = -rotation.2;

    let (sin_x, cos_x) = x.sin_cos();
    let (sin_y, cos_y) = y.sin_cos();
    let (sin_z, cos_z) = z.sin_cos();

    // optimized m(z) * m(y) * m(x)
    Matrix(
        cos_y * cos_z,
        sin_x * sin_y * cos_z - cos_x * sin_z,
        cos_x * sin_y * cos_z + sin_x * sin_z,
        cos_y * sin_z,
        sin_x * sin_y * sin_z + cos_x * cos_z,
        cos_x * sin_y * sin_z - sin_x * cos_z,
        -sin_y,
        sin_x * cos_y,
        cos_x * cos_y,
    )
}
