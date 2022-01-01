#![warn(clippy::all, clippy::cargo)]
mod activation;
mod header;

pub use activation::{
    read_activation, write_activation, ActivationStatus, ActivationType, AnimActivation,
};
pub use header::{read_save_header, write_save_header};
