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
    AnimActivation, AnimDef, AnimDefFile, AnimDefName, Execution, NamePad, NamePtr, NamePtrFlags,
    ResetState, SeqDef, SeqDefState,
};
use mech3ax_metadata_proc_macro::{Enum, Struct};
use mech3ax_timestamp::DateTime;
pub use si_script::{ObjectMotionSiFrame, RotateData, ScaleData, SiScript, TranslateData};
pub use support::{
    AnimRef, AnimRefCallAnimation, AnimRefCallObjectConnector, DynamicSoundRef, EffectRef,
    LightRef, NodeRef, ObjectRef, PufferRef, StaticSoundRef,
};

/// The mission is used for junk data (e.g. pointers).
///
/// For MW, "pre" refers to pre-v1.2 patch, and "post" post-v1.2 patch being applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Enum)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum AnimMission {
    /// * v1.0-us-pre
    /// * v1.1-us-pre
    /// * v1.0-us-post
    /// * v1.1-us-post
    MwC1V10,
    /// * v1.2-us-pre
    /// * v1.2-us-post
    /// * v1.0-de-pre
    /// * v1.0-de-post
    MwC1V12,
    /// * v1.0-us-pre
    /// * v1.1-us-pre
    /// * v1.0-us-post
    /// * v1.1-us-post
    MwC2V10,
    /// * v1.2-us-pre
    /// * v1.2-us-post
    MwC2V12,
    /// * v1.0-de-pre
    /// * v1.0-de-post
    MwC2V12De,
    /// * v1.0-us-pre
    /// * v1.1-us-pre
    MwC3V10,
    /// * v1.2-us-pre
    /// * v1.0-us-post
    /// * v1.1-us-post
    /// * v1.2-us-post
    MwC3V12,
    /// * v1.0-de-pre
    MwC3V12De,
    /// * v1.0-de-post
    MwC3V12DeP,
    /// * v1.0-us-pre
    /// * v1.1-us-pre
    /// * v1.2-us-pre
    /// * v1.0-us-post
    /// * v1.1-us-post
    /// * v1.2-us-post
    MwC4V10,
    /// * v1.0-de-pre
    /// * v1.0-de-post
    MwC4V12De,
    /// * v1.0-us-pre
    /// * v1.1-us-pre
    /// * v1.2-us-pre
    /// * v1.0-us-post
    /// * v1.1-us-post
    /// * v1.2-us-post
    MwC4bV10,
    /// * v1.0-de-pre
    /// * v1.0-de-post
    MwC4bV12De,
    /// * v1.0-us-pre
    /// * v1.1-us-pre
    /// * v1.2-us-pre
    /// * v1.0-us-post
    /// * v1.1-us-post
    /// * v1.2-us-post
    MwT1V10,
    /// * v1.0-de-pre
    /// * v1.0-de-post
    MwT1V12De,
    PmC1,
    PmC2,
    PmC3,
    PmC4,
    RcM01,
    RcM02,
    RcM03,
    RcM04,
    RcM05,
    RcM06,
    RcM07,
    RcM08,
    RcM09,
    RcM10,
    RcM11,
    RcM12,
    RcM13,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct AnimMetadata {
    /// The mission is used for junk data (e.g. pointers).
    pub mission: AnimMission,
    /// From `anim.zrd`: `GRAVITY`
    pub gravity: f32,
    /// The `anim.zbd` timestamp (PM only).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub datetime: Option<DateTime>,
    /// SI script names.
    pub script_names: Vec<String>,
    /// Anim definition names.
    pub anim_def_names: Vec<AnimDefName>,
    /// The list of animation definition files.
    ///
    /// This is not used by the engine.
    ///
    /// From `anim.zrd`:
    /// * `ANIMATION_DEFINITIONS`
    ///   * `ANIMATION_PATH` (maybe?)
    ///   * `ANIMATION_LIST`
    ///     * `ANIMATION_DEFINITION_FILE`
    pub anim_list: Vec<AnimDefFile>,
}
