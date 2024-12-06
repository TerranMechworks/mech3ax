mod activation_prereq;
mod anim_def;
pub mod events;
mod si_script;
mod support;

use ::serde::{Deserialize, Serialize};
pub use activation_prereq::{
    ActivationPrerequisite, PrerequisiteAnimation, PrerequisiteObject, PrerequisiteParent,
};
pub use anim_def::{
    AnimActivation, AnimDef, AnimDefFile, AnimPtr, Execution, NamePad, NamePtr, NamePtrFlags,
    ResetState, SeqActivation, SeqDef,
};
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_timestamp::DateTime;
pub use si_script::{ObjectMotionSiFrame, RotateData, ScaleData, SiScript, TranslateData};
pub use support::{
    AnimRef, AnimRefCallAnimation, AnimRefCallObjectConnector, DynamicSoundRef, EffectRef,
    LightRef, NodeRef, ObjectRef, PufferRef, StaticSoundRef,
};

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct AnimMetadata {
    /// The list of animation definition files.
    ///
    /// From `anim.zrd`:
    /// * `ANIMATION_DEFINITIONS`
    ///   * `ANIMATION_LIST`
    ///     * `ANIMATION_DEFINITION_FILE`
    pub anim_list: Vec<AnimDefFile>,
    pub anim_ptrs: Vec<AnimPtr>,
    pub scripts: Vec<SiScript>,
    // PM only
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub datetime: Option<DateTime>,
    pub gravity: f32,
    pub defs_ptr: u32,
    // PM only
    pub scripts_ptr: u32,
    pub world_ptr: u32,
    // PM only
    pub unk40: u32,
}
