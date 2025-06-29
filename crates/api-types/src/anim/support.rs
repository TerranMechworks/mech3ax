use crate::serde::bytes;
use crate::{fld, sum};

fld! {
    struct AnimRefCallAnimation: Val {
        name: String,
        #[serde(with = "bytes")]
        name_pad: Vec<u8>,
    }
}

fld! {
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

fld! {
    struct ObjectRef: Val {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        /// Ignored in PM.
        ptr: Option<u32>,
        /// `u16` in PM.
        flags: u32,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        /// Ignored in PM.
        flags_merged: Option<u32>,
        #[serde(with = "bytes")]
        /// The affine matrix cannot be serializes as is, because it contains bogus
        /// floats/values.
        affine: Vec<u8>,
    }
}

fld! {
    struct NodeRef: Val {
        name: String,
        ptr: u32,
    }
}

fld! {
    struct LightRef: Val {
        name: String,
        ptr: u32,
    }
}

fld! {
    struct PufferRef: Val {
        name: String,
        flags: u8,
        ptr: u32,
    }
}

fld! {
    struct DynamicSoundRef: Val {
        name: String,
        ptr: u32,
    }
}

fld! {
    struct StaticSoundRef: Val {
        name: String,
        #[serde(with = "bytes")]
        pad: Vec<u8>,
    }
}

fld! {
    struct EffectRef: Val {
        name: String,
        index: u32,
        #[serde(with = "bytes")]
        pad: Vec<u8>,
    }
}
