mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::saves::ActivationStatus;
use mech3ax_types::{impl_as_bytes, Ascii, Maybe};
pub use read::read_activation;
pub use write::write_activation;

const VALUES_SIZE: usize = 9 * 4;

type Status = Maybe<u8, ActivationStatus>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimActivationC {
    pub type_: i32,                // 00
    pub unk04: i32,                // 04
    pub name: Ascii<32>,           // 08
    pub node_index: i32,           // 40
    pub values: [u8; VALUES_SIZE], // 44
    pub unk80: u32,                // 80
    pub status: Status,            // 84
    pub count: u8,                 // 85
    pub unk86: u8,                 // 86
    pub unk87: u8,                 // 87
}
impl_as_bytes!(AnimActivationC, 88);
