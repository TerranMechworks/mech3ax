#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod activation;
mod header;

pub use activation::{
    read_activation, write_activation, ActivationStatus, ActivationType, AnimActivation,
};
pub use header::{read_save_header, write_save_header};
