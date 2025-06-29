use crate::serde::bytes;
use crate::{api, sum};

api! {
    struct AnimRefCallAnimation: Val {
        name: String,
        #[serde(with = "bytes")]
        name_pad: Vec<u8>,
    }
}

api! {
    struct AnimRefCallObjectConnector: Val {
        name: String,
        #[serde(with = "bytes")]
        name_pad: Vec<u8>,
        local_name: String,
        #[serde(with = "bytes")]
        local_name_pad: Vec<u8>,
    }
}

sum! {
    enum AnimRef {
        CallAnimation(AnimRefCallAnimation),
        CallObjectConnector(AnimRefCallObjectConnector),
    }
}

impl AnimRef {
    #[inline]
    pub fn name(&self) -> &String {
        match self {
            Self::CallAnimation(inner) => &inner.name,
            Self::CallObjectConnector(inner) => &inner.name,
        }
    }
}

api! {
    struct ObjectRef: Val {
        name: String,
        /// Ignored in PM.
        ptr: Option<u32> = { None },
        /// `u16` in PM.
        flags: u32,
        /// Ignored in PM.
        flags_merged: Option<u32> = { None },
        /// The affine matrix cannot be serializes as is, because it contains bogus
        /// floats/values.
        #[serde(with = "bytes")]
        affine: Vec<u8>,
    }
}

api! {
    struct NodeRef: Val {
        name: String,
        ptr: u32,
    }
}

api! {
    struct LightRef: Val {
        name: String,
        ptr: u32,
    }
}

api! {
    struct PufferRef: Val {
        name: String,
        flags: u8,
        ptr: u32,
    }
}

api! {
    struct DynamicSoundRef: Val {
        name: String,
        ptr: u32,
    }
}

api! {
    struct StaticSoundRef: Val {
        name: String,
        #[serde(with = "bytes")]
        pad: Vec<u8>,
    }
}

api! {
    struct EffectRef: Val {
        name: String,
        index: u32,
        #[serde(with = "bytes")]
        pad: Vec<u8>,
    }
}
