mod read;
mod write;

pub(crate) use read::{read_reset_state, read_sequence_defs};
pub(crate) use write::{write_reset_state, write_sequence_defs};

const SEQ_ACTIVATION_ON_CALL: u32 = 0x0303;
const SEQ_ACTIVATION_INITIAL: u32 = 0;
