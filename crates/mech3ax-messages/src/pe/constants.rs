bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ImageFileFlags: u16 {
        const RELOCS_STRIPPED = 0x0001;
        const EXECUTABLE_IMAGE = 0x0002;
        const LINE_NUMS_STRIPPED = 0x0004;
        const LOCAL_SYMS_STRIPPED = 0x0008;
        const AGGRESIVE_WS_TRIM = 0x0010;
        const LARGE_ADDRESS_AWARE = 0x0020;
        const MACHINE_16BIT = 0x0040;
        const BYTES_REVERSED_LO = 0x0080;
        const MACHINE_32BIT = 0x0100;
        const DEBUG_STRIPPED = 0x0200;
        const REMOVABLE_RUN_FROM_SWAP = 0x0400;
        const NET_RUN_FROM_SWAP = 0x0800;
        const SYSTEM = 0x1000;
        const DLL = 0x2000;
        const UP_SYSTEM_ONLY = 0x4000;
        const BYTES_REVERSED_HI = 0x8000;
    }
}

pub const IMAGE_DIRECTORY_ENTRY_RESOURCE: usize = 2;
