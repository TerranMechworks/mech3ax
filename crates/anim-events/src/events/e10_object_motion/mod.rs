mod ng;
mod rc;

use super::{EventMw, EventPm, EventRc};
use mech3ax_types::{bitflags, Maybe};

bitflags! {
    struct ObjectMotionFlags: u32 {
        /// Reader: `GRAVITY: [DEFAULT]`
        /// Reader: `GRAVITY: [LOCAL, <value>?, ...]`
        /// Reader: `GRAVITY: [COMPLEX, <value>?, ...]`
        const GRAVITY = 1 << 0;                     // 0x00001
        /// Reader: `IMPACT_FORCE: [ON]` (not seen in RC)
        const IMPACT_FORCE = 1 << 1;                // 0x00002
        /// Reader: `TRANSLATION: [xz, y, delta?, initial?]` ?
        const TRANSLATION = 1 << 2;                 // 0x00004
        /// Reader: `TRANSLATION_RANGE_MIN`/`TRANSLATION_RANGE_MAX`
        const TRANSLATION_RANGE_MIN = 1 << 3;           // 0x00008
        /// Reader: `TRANSLATION_RANGE_MIN`/`TRANSLATION_RANGE_MAX`
        ///
        /// Note: this flag is not required for the engine to use the ranges
        const TRANSLATION_RANGE_MAX = 1 << 4;             // 0x00010
        /// Reader: `XYZ_ROTATION: [?, ?, ?, ?, ?, ?]`
        const XYZ_ROTATION = 1 << 5;                // 0x00020
        /// Reader: `FORWARD_ROTATION: [DISTANCE, <?>, <?>]` (not seen in RC)
        const FORWARD_ROTATION_DISTANCE = 1 << 6;   // 0x00040
        /// Reader: `FORWARD_ROTATION: [TIME, <?>, <?>]`
        const FORWARD_ROTATION_TIME = 1 << 7;       // 0x00080
        /// Reader: `SCALE: [xi, yi, zi, xd, yd, zd]`?
        const SCALE = 1 << 8;                       // 0x00100
        /// Not in reader
        ///
        /// This seems to be a delta value.
        const MORPH = 1 << 9;                       // 0x00200
        /// Reader: `RUN_TIME: [<run_time>]`
        const RUN_TIME = 1 << 10;                   // 0x00400
        /// Reader: `BOUNCE_SEQUENCE: [<seq_name>]`
        /// Reader: `BOUNCE_SEQUENCE_WATER: [<seq_name>]`
        /// Reader: `BOUNCE_SEQUENCE_LAVA: [<seq_name>]`
        const BOUNCE_SEQUENCE = 1 << 11;            // 0x00800
        /// Reader: `BOUNCE_SOUND: [NAME: [<name>], FULL_VOLUME_VELOCITY: [<velocity>]]`
        const BOUNCE_SOUND = 1 << 12;               // 0x01000
        /// Reader: `GRAVITY: [COMPLEX, <value>?]`
        const GRAVITY_COMPLEX = 1 << 13;            // 0x02000
        /// Reader: `GRAVITY: [..., NO_ALTITUDE]` (not RC)
        const GRAVITY_NO_ALTITUDE = 1 << 14;        // 0x04000
    }
}

impl ObjectMotionFlags {
    const TRANSLATION_RANGE: Self = Self::from_bits_truncate(
        Self::TRANSLATION_RANGE_MIN.bits() | Self::TRANSLATION_RANGE_MAX.bits(),
    );
    // const TRANSLATION_ANY: Self =
    //     Self::from_bits_truncate(Self::TRANSLATION.bits() | Self::TRANSLATION_RANGE.bits());
    // const FORWARD_ROTATION_ANY: Self = Self::from_bits_truncate(
    //     Self::FORWARD_ROTATION_DISTANCE.bits() | Self::FORWARD_ROTATION_TIME.bits(),
    // );
}

type Flags = Maybe<u32, ObjectMotionFlags>;
