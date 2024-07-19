#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod pixel_ops;

pub use pixel_ops::{
    pal8to888, pal8to888a, rgb565to888, rgb565to888a, rgb888ato565, rgb888atopal8, rgb888to565,
    rgb888topal8, simple_alpha,
};
