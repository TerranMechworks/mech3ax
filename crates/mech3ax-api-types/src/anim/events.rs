use crate::serde::{base64, bool_false};
use crate::types::{Color, Quaternion, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Enum, RefStruct, Union, ValStruct};
use num_derive::FromPrimitive;

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct AtNode {
    pub node: String,
    pub translation: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct StopAnimation {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ResetAnimation {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct InvalidateAnimation {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct CallAnimationAtNode {
    pub node: String,
    pub translation: Option<Vec3>,
    pub rotation: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct CallAnimationWithNode {
    pub node: String,
    pub translation: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct CallAnimationTargetNode {
    pub operand_node: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum CallAnimationParameters {
    AtNode(CallAnimationAtNode),
    WithNode(CallAnimationWithNode),
    TargetNode(CallAnimationTargetNode),
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct CallAnimation {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub wait_for_completion: Option<u16>,
    pub parameters: CallAnimationParameters,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct CallObjectConnector {
    pub node: String,
    pub from_node: String,
    pub to_node: String,
    pub to_pos: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct Loop {
    pub start: i32,
    pub loop_count: i32,
}

// TODO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum If {
    RandomWeight(f32),
    PlayerRange(f32),
    AnimationLod(u32),
    HwRender(bool),
    PlayerFirstPerson(bool),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ElseIf {
    RandomWeight(f32),
    PlayerRange(f32),
    AnimationLod(u32),
    HwRender(bool),
    PlayerFirstPerson(bool),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Else;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndIf;

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct Callback {
    pub value: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct DetonateWeapon {
    pub name: String,
    pub at_node: AtNode,
}

#[derive(Debug, Serialize, Deserialize, Clone, ValStruct)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct FrameBufferEffectColor {
    pub from: Rgba,
    pub to: Rgba,
    pub runtime: f32,
    // this value can be safely ignored, but is required for binary accuracy
    #[serde(skip_serializing_if = "bool_false", default)]
    pub fudge_alpha: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Enum)]
#[repr(u32)]
pub enum FogType {
    Off = 0,
    Linear = 1,
    Exponential = 2,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct FogState {
    pub name: String,
    pub fog_type: FogType,
    pub color: Color,
    pub altitude: Range,
    pub range: Range,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct LightAnimation {
    pub name: String,
    pub range: Range,
    pub color: Color,
    pub runtime: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct LightState {
    pub name: String,
    pub active_state: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub directional: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub saturated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub subdivide: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub static_: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<AtNode>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ambient: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub diffuse: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectActiveState {
    pub node: String,
    pub state: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectAddChild {
    // in the reader zbd, both values are fused into a list (PARENT_CHILD)
    pub parent: String,
    pub child: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectConnector {
    pub node: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub from_node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub to_node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub from_pos: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub to_pos: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_length: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectCycleTexture {
    pub node: String,
    pub reset: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct FloatFromTo {
    pub from: f32,
    pub to: f32,
    pub delta: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct Vec3FromTo {
    pub from: Vec3,
    pub to: Vec3,
    pub delta: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectMotionFromTo {
    pub node: String,
    pub run_time: f32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub morph: Option<FloatFromTo>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translate: Option<Vec3FromTo>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotate: Option<Vec3FromTo>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale: Option<Vec3FromTo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct TranslateData {
    pub value: Vec3,
    #[serde(with = "base64")]
    pub unk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct RotateData {
    pub value: Quaternion,
    #[serde(with = "base64")]
    pub unk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ScaleData {
    pub value: Vec3,
    #[serde(with = "base64")]
    pub unk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectMotionSiFrame {
    pub start_time: f32,
    pub end_time: f32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translation: Option<TranslateData>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotation: Option<RotateData>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale: Option<ScaleData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectMotionSiScript {
    pub node: String,
    pub frames: Vec<ObjectMotionSiFrame>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Enum)]
pub enum GravityMode {
    Local,
    Complex,
    NoAltitude,
}

#[derive(Debug, Serialize, Deserialize, Clone, ValStruct)]
pub struct Gravity {
    pub mode: GravityMode,
    pub value: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ValStruct)]
pub struct ForwardRotationTime {
    pub v1: f32,
    pub v2: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ValStruct)]
pub struct ForwardRotationDistance {
    pub v1: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum ForwardRotation {
    Time(ForwardRotationTime),
    Distance(ForwardRotationDistance),
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct BounceSound {
    pub name: String,
    pub volume: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectMotion {
    pub node: String,
    pub impact_force: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gravity: Option<Gravity>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translation_range_min: Option<Quaternion>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translation_range_max: Option<Quaternion>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translation: Option<(Vec3, Vec3, Vec3)>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub forward_rotation: Option<ForwardRotation>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub xyz_rotation: Option<(Vec3, Vec3)>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale: Option<(Vec3, Vec3)>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bounce_sequence: Option<(Option<String>, Option<String>, Option<String>)>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bounce_sound: Option<BounceSound>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub runtime: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectOpacityFromTo {
    pub node: String,
    pub opacity_from: (f32, i16),
    pub opacity_to: (f32, i16),
    pub runtime: f32,
    // this value can be safely ignored, but is required for binary accuracy
    #[serde(skip_serializing_if = "bool_false", default)]
    pub fudge: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectOpacityState {
    pub node: String,
    pub is_set: bool,
    pub state: bool,
    pub opacity: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum RotateState {
    Absolute(Vec3),
    AtNodeXYZ,
    AtNodeMatrix,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectRotateState {
    pub node: String,
    pub rotate: RotateState,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectScaleState {
    pub node: String,
    pub scale: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct ObjectTranslateState {
    pub node: String,
    pub translate: Vec3,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Enum)]
pub enum IntervalType {
    Unset,
    Time,
    Distance,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct Interval {
    pub interval_type: IntervalType,
    pub interval_value: f32,
    pub flag: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct PufferStateCycleTextures {
    pub texture1: Option<String>,
    pub texture2: Option<String>,
    pub texture3: Option<String>,
    pub texture4: Option<String>,
    pub texture5: Option<String>,
    pub texture6: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct PufferState {
    pub name: String,
    pub state: bool,
    pub translate: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub active_state: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<AtNode>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub local_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub world_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub min_random_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_random_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub world_acceleration: Option<Vec3>,
    pub interval: Interval,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub lifetime_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub start_age_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub deviation_distance: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fade_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub friction: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub textures: Option<PufferStateCycleTextures>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub growth_factor: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct CallSequence {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct StopSequence {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct SoundNode {
    pub name: String,
    pub active_state: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<AtNode>,
}

#[derive(Debug, Serialize, Deserialize, Clone, RefStruct)]
pub struct Sound {
    pub name: String,
    pub at_node: AtNode,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventData {
    Sound(Sound),
    SoundNode(SoundNode),
    LightState(LightState),
    LightAnimation(LightAnimation),
    ObjectActiveState(ObjectActiveState),
    ObjectTranslateState(ObjectTranslateState),
    ObjectScaleState(ObjectScaleState),
    ObjectRotateState(ObjectRotateState),
    ObjectMotion(ObjectMotion),
    ObjectMotionFromTo(ObjectMotionFromTo),
    ObjectMotionSIScript(ObjectMotionSiScript),
    ObjectOpacityState(ObjectOpacityState),
    ObjectOpacityFromTo(ObjectOpacityFromTo),
    ObjectAddChild(ObjectAddChild),
    ObjectCycleTexture(ObjectCycleTexture),
    ObjectConnector(ObjectConnector),
    CallObjectConnector(CallObjectConnector),
    CallSequence(CallSequence),
    StopSequence(StopSequence),
    CallAnimation(CallAnimation),
    StopAnimation(StopAnimation),
    ResetAnimation(ResetAnimation),
    InvalidateAnimation(InvalidateAnimation),
    FogState(FogState),
    Loop(Loop),
    If(If),
    Else(Else),
    Elif(ElseIf),
    Endif(EndIf),
    Callback(Callback),
    FrameBufferEffectColorFromTo(FrameBufferEffectColor),
    DetonateWeapon(DetonateWeapon),
    PufferState(PufferState),
}

#[derive(Debug, Serialize, Deserialize, FromPrimitive, Clone, Copy, PartialEq, Enum)]
#[repr(u8)]
pub enum StartOffset {
    Animation = 1,
    Sequence = 2,
    Event = 3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub data: EventData,
    pub start: Option<(StartOffset, f32)>,
}