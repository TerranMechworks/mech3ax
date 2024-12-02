#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod common;
pub mod mw;
pub mod pm;
pub mod rc;

use mech3ax_types::Hex;

#[allow(clippy::excessive_precision)]
const GRAVITY: f32 = -9.800000190734863;
const SIGNATURE: Hex<u32> = Hex(0x08170616);

const VERSION_RC: u32 = 28;
const VERSION_MW: u32 = 39;
const VERSION_PM: u32 = 50;
