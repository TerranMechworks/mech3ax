use crate::serde::{bool_false, bytes};
use crate::{Color, Quaternion, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Enum, Struct, Union};
use mech3ax_types::primitive_enum;
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct AtNode {
    pub node: String,
    pub translation: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct StopAnimation {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ResetAnimation {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct InvalidateAnimation {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallAnimationAtNode {
    pub node: String,
    pub translation: Option<Vec3>,
    pub rotation: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallAnimationWithNode {
    pub node: String,
    pub translation: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallAnimation {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub wait_for_completion: Option<u16>,
    pub parameters: CallAnimationParameters,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallObjectConnector {
    pub node: String,
    pub from_node: String,
    pub to_node: String,
    pub to_pos: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Loop {
    pub start: i32,
    pub loop_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Struct)]
#[dotnet(val_struct)]
pub struct RandomWeightCond {
    pub value: f32,
}

impl From<f32> for RandomWeightCond {
    fn from(value: f32) -> Self {
        Self { value }
    }
}

impl Deref for RandomWeightCond {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Struct)]
#[dotnet(val_struct)]
pub struct PlayerRangeCond {
    pub value: f32,
}

impl From<f32> for PlayerRangeCond {
    fn from(value: f32) -> Self {
        Self { value }
    }
}

impl Deref for PlayerRangeCond {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Struct)]
#[dotnet(val_struct)]
pub struct AnimationLodCond {
    pub value: u32,
}

impl From<u32> for AnimationLodCond {
    fn from(value: u32) -> Self {
        Self { value }
    }
}

impl Deref for AnimationLodCond {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Struct)]
#[dotnet(val_struct)]
pub struct HwRenderCond {
    pub value: bool,
}

impl From<bool> for HwRenderCond {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl Deref for HwRenderCond {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Struct)]
#[dotnet(val_struct)]
pub struct PlayerFirstPersonCond {
    pub value: bool,
}

impl From<bool> for PlayerFirstPersonCond {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl Deref for PlayerFirstPersonCond {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum If {
    RandomWeight(RandomWeightCond),
    PlayerRange(PlayerRangeCond),
    AnimationLod(AnimationLodCond),
    HwRender(HwRenderCond),
    PlayerFirstPerson(PlayerFirstPersonCond),
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum ElseIf {
    RandomWeight(RandomWeightCond),
    PlayerRange(PlayerRangeCond),
    AnimationLod(AnimationLodCond),
    HwRender(HwRenderCond),
    PlayerFirstPerson(PlayerFirstPersonCond),
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Else {}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct EndIf {}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Callback {
    pub value: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct DetonateWeapon {
    pub name: String,
    pub at_node: AtNode,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FrameBufferEffectColor {
    pub from: Rgba,
    pub to: Rgba,
    pub runtime: f32,
    // this value can be safely ignored, but is required for binary accuracy
    #[serde(skip_serializing_if = "bool_false", default)]
    pub fudge_alpha: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Enum)]
#[repr(u32)]
pub enum FogType {
    Off = 0,
    Linear = 1,
    Exponential = 2,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FogState {
    pub name: String,
    pub fog_type: FogType,
    pub color: Color,
    pub altitude: Range,
    pub range: Range,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct LightAnimation {
    pub name: String,
    pub range: Range,
    pub color: Color,
    pub runtime: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectActiveState {
    pub node: String,
    pub state: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectAddChild {
    // in the reader zbd, both values are fused into a list (PARENT_CHILD)
    pub parent: String,
    pub child: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectCycleTexture {
    pub node: String,
    pub reset: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FloatFromTo {
    pub from: f32,
    pub to: f32,
    pub delta: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Vec3FromTo {
    pub from: Vec3,
    pub to: Vec3,
    pub delta: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct TranslateData {
    pub value: Vec3,
    #[serde(with = "bytes")]
    pub unk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct RotateData {
    pub value: Quaternion,
    #[serde(with = "bytes")]
    pub unk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ScaleData {
    pub value: Vec3,
    #[serde(with = "bytes")]
    pub unk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectMotionSiScript {
    pub node: String,
    pub frames: Vec<ObjectMotionSiFrame>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Enum)]
pub enum GravityMode {
    Local,
    Complex,
    NoAltitude,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct Gravity {
    pub mode: GravityMode,
    pub value: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct ForwardRotationTime {
    pub v1: f32,
    pub v2: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct ForwardRotationDistance {
    pub v1: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum ForwardRotation {
    Time(ForwardRotationTime),
    Distance(ForwardRotationDistance),
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct BounceSequence {
    pub seq_name0: Option<String>,
    pub seq_name1: Option<String>,
    pub seq_name2: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectMotionTranslation {
    pub delta: Vec3,
    pub initial: Vec3,
    pub unk: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct XyzRotation {
    pub value: Vec3,
    pub unk: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectMotionScale {
    pub value: Vec3,
    pub unk: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct BounceSound {
    pub name: String,
    pub volume: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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
    pub translation: Option<ObjectMotionTranslation>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub forward_rotation: Option<ForwardRotation>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub xyz_rotation: Option<XyzRotation>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale: Option<ObjectMotionScale>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bounce_sequence: Option<BounceSequence>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bounce_sound: Option<BounceSound>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub runtime: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct ObjectOpacity {
    pub value: f32,
    pub state: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectOpacityFromTo {
    pub node: String,
    pub opacity_from: ObjectOpacity,
    pub opacity_to: ObjectOpacity,
    pub runtime: f32,
    // this value can be safely ignored, but is required for binary accuracy
    #[serde(skip_serializing_if = "bool_false", default)]
    pub fudge: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectRotateState {
    pub node: String,
    pub rotate: RotateState,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectScaleState {
    pub node: String,
    pub scale: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectTranslateState {
    pub node: String,
    pub translate: Vec3,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Enum)]
pub enum IntervalType {
    Unset,
    Time,
    Distance,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Interval {
    pub interval_type: IntervalType,
    pub interval_value: f32,
    pub flag: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct PufferStateCycleTextures {
    pub texture1: Option<String>,
    pub texture2: Option<String>,
    pub texture3: Option<String>,
    pub texture4: Option<String>,
    pub texture5: Option<String>,
    pub texture6: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallSequence {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct StopSequence {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct SoundNode {
    pub name: String,
    pub active_state: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<AtNode>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Sound {
    pub name: String,
    pub at_node: AtNode,
}

#[derive(Debug, Serialize, Deserialize, Union)]
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

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum StartOffset: u8 {
        Animation = 1,
        Sequence = 2,
        Event = 3,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct EventStart {
    pub offset: StartOffset,
    pub time: f32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Event {
    pub data: EventData,
    pub start: Option<EventStart>,
}
