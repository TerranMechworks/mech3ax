use crate::serde::bytes;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Struct, Union};

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct AnimRefCallAnimation {
    pub name: String,
    #[serde(with = "bytes")]
    pub name_pad: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct AnimRefCallObjectConnector {
    pub name: String,
    #[serde(with = "bytes")]
    pub name_pad: Vec<u8>,
    pub local_name: String,
    #[serde(with = "bytes")]
    pub local_name_pad: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum AnimRef {
    CallAnimation(AnimRefCallAnimation),
    CallObjectConnector(AnimRefCallObjectConnector),
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

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct ObjectRef {
    pub name: String,
    #[serde(with = "bytes")]
    pub unk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct NodeRef {
    pub name: String,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct LightRef {
    pub name: String,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct PufferRef {
    pub name: String,
    pub flags: u8,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct DynamicSoundRef {
    pub name: String,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct StaticSoundRef {
    pub name: String,
    #[serde(with = "bytes")]
    pub pad: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct EffectRef {
    pub name: String,
    pub unk32: u32,
    #[serde(with = "bytes")]
    pub pad: Vec<u8>,
}
