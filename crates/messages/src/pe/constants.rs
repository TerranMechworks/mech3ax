use mech3ax_types::{Maybe, bitflags};

bitflags! {
    pub(crate) struct ImageFileFlags: u16 {
        const RELOCS_STRIPPED = 1 << 0; // 0x0001
        const EXECUTABLE_IMAGE = 1 << 1; // 0x0002
        const LINE_NUMS_STRIPPED = 1 << 2; // 0x0004
        const LOCAL_SYMS_STRIPPED = 1 << 3; // 0x0008
        const AGGRESIVE_WS_TRIM = 1 << 4; // 0x0010
        const LARGE_ADDRESS_AWARE = 1 << 5; // 0x0020
        const MACHINE_16BIT = 1 << 6; // 0x0040
        const BYTES_REVERSED_LO = 1 << 7; // 0x0080
        const MACHINE_32BIT = 1 << 8; // 0x0100
        const DEBUG_STRIPPED = 1 << 9; // 0x0200
        const REMOVABLE_RUN_FROM_SWAP = 1 << 10; // 0x0400
        const NET_RUN_FROM_SWAP = 1 << 11; // 0x0800
        const SYSTEM = 1 << 12; // 0x1000
        const DLL = 1 << 13; // 0x2000
        const UP_SYSTEM_ONLY = 1 << 14; // 0x4000
        const BYTES_REVERSED_HI = 1 << 15; // 0x8000
    }
}

pub(crate) type Flags = Maybe<u16, ImageFileFlags>;

pub(crate) const IMAGE_DIRECTORY_ENTRY_RESOURCE: usize = 2;
