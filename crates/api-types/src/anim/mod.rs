#![allow(clippy::mistyped_literal_suffixes)]
mod activation_prereq;
mod anim_def;
pub mod events;
mod si_script;
mod support;

use crate::{api, num};
pub use activation_prereq::{
    ActivationPrerequisite, PrerequisiteAnimation, PrerequisiteObject, PrerequisiteParent,
};
pub use anim_def::{
    AnimActivation, AnimDef, AnimDefFile, AnimDefPtrs, Execution, NamePad, NamePtr, NamePtrFlags,
    ResetState, SeqDef, SeqDefState,
};
use mech3ax_timestamp::DateTime;
pub use si_script::{ObjectMotionSiFrame, RotateData, ScaleData, SiScript, TranslateData};
pub use support::{
    AnimRef, AnimRefCallAnimation, AnimRefCallObjectConnector, DynamicSoundRef, EffectRef,
    LightRef, NodeRef, ObjectRef, PufferRef, StaticSoundRef,
};

num! {
    /// The mission is used for junk data (e.g. pointers).
    ///
    /// For MW, "pre" refers to pre-v1.2 patch, and "post" post-v1.2 patch being applied.
    enum AnimMission {
        Unknown = 0,
        /// * v1.0-us-pre
        /// * v1.1-us-pre
        /// * v1.0-us-post
        /// * v1.1-us-post
        MwC1V10 = 1_01_10,
        /// * v1.2-us-pre
        /// * v1.2-us-post
        /// * v1.0-de-pre
        /// * v1.0-de-post
        MwC1V12 = 1_01_12,
        /// * v1.0-us-pre
        /// * v1.1-us-pre
        /// * v1.0-us-post
        /// * v1.1-us-post
        MwC2V10 = 1_02_10,
        /// * v1.2-us-pre
        /// * v1.2-us-post
        MwC2V12 = 1_02_12,
        /// * v1.0-de-pre
        /// * v1.0-de-post
        MwC2V12De = 1_02_32,
        /// * v1.0-us-pre
        /// * v1.1-us-pre
        MwC3V10 = 1_03_10,
        /// * v1.2-us-pre
        /// * v1.0-us-post
        /// * v1.1-us-post
        /// * v1.2-us-post
        MwC3V12 = 1_03_12,
        /// * v1.0-de-pre
        MwC3V12De = 1_03_32,
        /// * v1.0-de-post
        MwC3V12DeP = 1_03_33,
        /// * v1.0-us-pre
        /// * v1.1-us-pre
        /// * v1.2-us-pre
        /// * v1.0-us-post
        /// * v1.1-us-post
        /// * v1.2-us-post
        MwC4V10 = 1_04_10,
        /// * v1.0-de-pre
        /// * v1.0-de-post
        MwC4V12De = 1_04_32,
        /// * v1.0-us-pre
        /// * v1.1-us-pre
        /// * v1.2-us-pre
        /// * v1.0-us-post
        /// * v1.1-us-post
        /// * v1.2-us-post
        MwC4bV10 = 1_05_10,
        /// * v1.0-de-pre
        /// * v1.0-de-post
        MwC4bV12De = 1_07_32,
        /// * v1.0-us-pre
        /// * v1.1-us-pre
        /// * v1.2-us-pre
        /// * v1.0-us-post
        /// * v1.1-us-post
        /// * v1.2-us-post
        MwT1V10 = 1_06_10,
        /// * v1.0-de-pre
        /// * v1.0-de-post
        MwT1V12De = 1_06_32,
        PmC1 = 2_01_00,
        PmC2 = 2_02_00,
        PmC3 = 2_03_00,
        PmC4 = 2_04_00,
        RcM01 = 3_01_00,
        RcM02 = 3_02_00,
        RcM03 = 3_03_00,
        RcM04 = 3_04_00,
        RcM05 = 3_05_00,
        RcM06 = 3_06_00,
        RcM07 = 3_07_00,
        RcM08 = 3_08_00,
        RcM09 = 3_09_00,
        RcM10 = 3_10_00,
        RcM11 = 3_11_00,
        RcM12 = 3_12_00,
        RcM13 = 3_13_00,
    }
}

api! {
    struct AnimMetadata {
        /// The mission is used for junk data (e.g. pointers).
        mission: AnimMission,
        /// From `anim.zrd`: `GRAVITY`
        gravity: f32,
        /// The `anim.zbd` timestamp (PM only).
        #[serde(skip_serializing_if = "Option::is_none", default)]
        datetime: Option<DateTime> = { None },
        /// Anim definition names.
        anim_def_names: Vec<String>,
        /// SI script names.
        script_names: Vec<String>,
        /// The list of animation definition files.
        ///
        /// This is not used by the engine.
        ///
        /// From `anim.zrd`:
        /// * `ANIMATION_DEFINITIONS`
        ///   * `ANIMATION_PATH` (maybe?)
        ///   * `ANIMATION_LIST`
        ///     * `ANIMATION_DEFINITION_FILE`
        anim_list: Vec<AnimDefFile>,
    }
}
