mod mw;

const SIGNATURE: u32 = 0x02971222;

#[allow(unused)]
const VERSION_RC: u32 = 15;
const VERSION_MW: u32 = 27;
#[allow(unused)]
const VERSION_PM: u32 = 41;
#[allow(unused)]
const VERSION_CS: u32 = 42;

pub use mw::{read_gamez_mw, write_gamez_mw};
