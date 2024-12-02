use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Struct, Union};

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct PrerequisiteAnimation {
    pub name: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct PrerequisiteObject {
    pub name: String,
    pub required: bool,
    pub active: bool,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct PrerequisiteParent {
    pub name: String,
    pub required: bool,
    pub active: bool,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum ActivationPrerequisite {
    Animation(PrerequisiteAnimation),
    Parent(PrerequisiteParent),
    Object(PrerequisiteObject),
}
