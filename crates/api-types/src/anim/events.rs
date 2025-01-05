use crate::serde::bytes;
use crate::{Color, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::{Enum, Struct, Union};
use mech3ax_types::{impl_as_bytes, primitive_enum};

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
    pub start: Option<EventStart>,
    pub data: EventData,
}

/// AT_NODE
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct AtNode {
    /// node name
    ///
    /// Warning: Whether INPUT_NODE is allowed depends on the parent struct!
    pub name: String,
    pub pos: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum Translate {
    Absolute(Vec3),
    AtNode(AtNode),
}

/// SOUND Index: 01
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Sound {
    /// NAME (static sound name)
    pub name: String,
    /// AT_NODE (node name)
    pub at_node: Option<AtNode>,
}

/// SOUND_NODE Index: 02
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct SoundNode {
    /// NAME (sound node name)
    pub name: String,
    /// ACTIVE_STATE
    pub active_state: bool,
    /// AT_NODE TODO (node name)
    pub translate: Option<Translate>,
}

/// EFFECT Index: 03
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Effect {
    /// NAME (effect name)
    pub name: String,
    /// AT_NODE (node name or INPUT_NODE)
    pub at_node: AtNode,
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum LightType: u32 {
        Directed = 0,
        PointSource = 1,
    }
}

/// LIGHT_STATE Index: 04
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct LightState {
    /// NAME (light name)
    pub name: String,
    /// ACTIVE_STATE
    pub active_state: bool,
    /// TYPE
    pub type_: LightType,
    /// AT_NODE TODO (node name or INPUT_NODE)
    pub translate: Option<Translate>,
    /// DIRECTIONAL
    pub directional: Option<bool>,
    /// SATURATED
    pub saturated: Option<bool>,
    /// SUBDIVIDE
    pub subdivide: Option<bool>,
    /// LIGHTMAP
    pub lightmap: Option<bool>,
    /// STATIC
    pub static_: Option<bool>,
    /// BICOLORED (not in reader)
    pub bicolored: Option<bool>,
    /// ORIENTATION (not in reader)
    pub orientation: Option<Vec3>,
    /// RANGE
    pub range: Option<Range>,
    /// COLOR
    pub color: Option<Color>,
    /// AMBIENT_COLOR (not in reader)
    pub ambient_color: Option<Color>,
    /// AMBIENT
    pub ambient: Option<f32>,
    /// DIFFUSE
    pub diffuse: Option<f32>,
}

/// LIGHT_ANIMATION Index: 05
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct LightAnimation {
    /// NAME (light name)
    pub name: String,
    /// RANGE
    pub range: Range,
    /// COLOR
    pub color: Color,
    /// RUN_TIME
    pub run_time: f32,
    /// RANGE (second set of values?)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub range_alt: Option<Range>,
}

/// OBJECT_ACTIVE_STATE Index: 06
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectActiveState {
    /// NAME (node name or INPUT_NODE)
    pub node: String,
    /// STATE
    pub state: bool,
}

/// OBJECT_TRANSLATE_STATE Index: 07
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectTranslateState {
    /// NAME (node name)
    pub node: String,
    /// STATE / RELATIVE
    pub relative: bool,
    /// STATE / RELATIVE
    pub state: Vec3,
    /// AT_NODE (node name or INPUT_NODE)
    pub at_node: Option<String>,
}

/// OBJECT_SCALE_STATE Index: 08
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectScaleState {
    /// NAME (node name)
    pub name: String,
    /// STATE
    pub state: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum RotateBasis {
    /// STATE
    Absolute,
    /// not in reader
    Relative,
    /// AT_NODE_MATRIX (node name or INPUT_NODE)
    ///
    /// Warning: Ignored for Camera nodes!
    AtNodeMatrix(String),
    /// AT_NODE_XYZ (node name or INPUT_NODE)
    ///
    /// Warning: Ignored for Camera nodes!
    AtNodeXYZ(String),
}

/// OBJECT_ROTATE_STATE Index: 09
/// Camera and Object3d nodes only!
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectRotateState {
    /// NAME (node name)
    pub name: String,
    /// STATE / AT_NODE_MATRIX / AT_NODE_XYZ
    pub state: Vec3,
    /// STATE / AT_NODE_MATRIX / AT_NODE_XYZ
    pub basis: RotateBasis,
}

/// GRAVITY
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Enum)]
pub enum GravityMode {
    /// LOCAL
    Local,
    /// COMPLEX
    Complex,
    /// NO_ALTITUDE?
    NoAltitude,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct Gravity {
    pub mode: GravityMode,
    pub value: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct TranslationRange {
    pub xz: Range,
    pub y: Range,
    pub delta: Range,
    pub initial: Range,
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

/// FORWARD_ROTATION
#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum ForwardRotation {
    /// TIME
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

/// BOUNCE_SOUND
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct BounceSound {
    /// NAME
    pub name: String,
    /// FULL_VOLUME_VELOCITY
    pub volume: f32,
}

/// OBJECT_MOTION Index: 10
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectMotion {
    /// NAME (node name)
    pub node: String,

    pub impact_force: bool,
    /// GRAVITY
    ///
    /// DEFAULT / LOCAL
    pub gravity: Option<Gravity>,
    /// TRANSLATION_RANGE
    pub translation_range: Option<TranslationRange>,
    /// TRANSLATION
    pub translation: Option<ObjectMotionTranslation>,
    /// FORWARD_ROTATION
    pub forward_rotation: Option<ForwardRotation>,
    /// XYZ_ROTATION
    pub xyz_rotation: Option<XyzRotation>,
    /// SCALE
    pub scale: Option<ObjectMotionScale>,
    /// BOUNCE_SEQUENCE
    pub bounce_sequence: Option<BounceSequence>,
    /// BOUNCE_SOUND
    pub bounce_sound: Option<BounceSound>,
    /// RUN_TIME
    pub run_time: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Vec3FromTo {
    pub from: Vec3,
    pub to: Vec3,
}

/// OBJECT_MOTION_FROM_TO Index: 11
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectMotionFromTo {
    /// NAME (node name)
    pub name: String,
    /// RUN_TIME
    pub run_time: f32,
    /// MORPH_FROM / MORPH_TO
    pub morph: Option<FloatFromTo>,
    /// TRANSLATE_FROM / TRANSLATE_TO
    ///
    /// Warning: Only applies to Camera/Object3D nodes!
    pub translate: Option<Vec3FromTo>,
    /// ROTATE_FROM / ROTATE_TO
    ///
    /// Warning: Only applies to Camera/Object3D nodes!
    pub rotate: Option<Vec3FromTo>,
    /// SCALE_FROM / SCALE_TO
    ///
    /// Warning: Only applies to Object3D nodes!
    pub scale: Option<Vec3FromTo>,
    /// Only used for binary accuracy.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translate_delta: Option<Vec3>,
    /// Only used for binary accuracy.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotate_delta: Option<Vec3>,
    /// Only used for binary accuracy.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale_delta: Option<Vec3>,
}

/// OBJECT_MOTION_SI_SCRIPT Index: 12
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectMotionSiScript {
    /// NAME (node name)
    pub name: String,
    /// SCRIPT_FILENAME
    pub index: u32,
}

/// OBJECT_OPACITY_STATE Index: 13
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectOpacityState {
    /// NAME (node name or INPUT_NODE)
    pub name: String,
    /// STATE / IsSet in interp
    pub state: bool,
    /// STATE
    pub opacity: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct ObjectOpacity {
    /// STATE
    pub opacity: f32,
    /// STATE / IsSet in interp
    pub state: Option<bool>,
}

/// OBJECT_OPACITY_FROM_TO Index: 14
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectOpacityFromTo {
    /// NAME (node name)
    pub name: String,
    /// OPACITY_FROM
    pub opacity_from: ObjectOpacity,
    /// OPACITY_TO
    pub opacity_to: ObjectOpacity,
    /// RUN_TIME
    pub run_time: f32,
    /// Only used for binary accuracy.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub opacity_delta: Option<f32>,
}

/// OBJECT_ADD_CHILD Index: 15
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectAddChild {
    /// PARENT_CHILD (node name)
    pub parent: String,
    /// PARENT_CHILD (node name)
    pub child: String,
}

/// OBJECT_DELETE_CHILD Index: 16
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectDeleteChild {
    /// PARENT_CHILD (node name)
    pub parent: String,
    /// PARENT_CHILD (node name)
    pub child: String,
}

/// OBJECT_CYCLE_TEXTURE Index: 17
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectCycleTexture {
    /// NAME (node name)
    pub name: String,
    /// RESET (0..6)
    pub reset: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum ObjectConnectorPos {
    /// FROM_POS / TO_POS
    Pos(Vec3),
    /// FROM_INPUT_POS / TO_INPUT_POS
    Input,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum ObjectConnectorTime {
    /// FROM_T / TO_T
    Value(f32),
    /// FROM_T_START + FROM_T_END / TO_T_START + FROM_T_END
    Range(Range),
}

/// OBJECT_CONNECTOR Index: 18
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectConnector {
    /// NAME
    pub name: String,
    /// FROM_NODE / FROM_INPUT_NODE (node name or INPUT_NODE)
    pub from_node: Option<String>,
    /// TO_NODE / TO_INPUT_NODE (node name or INPUT_NODE)
    pub to_node: Option<String>,
    /// FROM_POS / FROM_INPUT_POS
    ///
    /// Warning: This is ignored if `from_node` is set!
    pub from_pos: Option<ObjectConnectorPos>,
    /// TO_POS / TO_INPUT_POS
    ///
    /// Warning: This is ignored if `to_node` is set!
    pub to_pos: Option<ObjectConnectorPos>,
    /// FROM_T_START + FROM_T_END / FROM_T
    pub from_t: Option<ObjectConnectorTime>,
    /// TO_T_START + FROM_T_END / TO_T
    pub to_t: Option<ObjectConnectorTime>,
    /// RUN_TIME
    pub run_time: f32,
    /// MAX_LENGTH
    pub max_length: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallObjectConnectorTarget {
    /// FROM_NODE / TO_NODE (node name or INPUT_NODE)
    pub name: String,
    /// FROM_NODE_POS / TO_NODE_POS (node name or INPUT_NODE)
    ///
    /// Warning: This overrides the position and unsets the node!
    pub pos: bool,
}

/// CALL_OBJECT_CONNECTOR Index: 19
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallObjectConnector {
    /// NAME (anim name)
    pub name: String,
    /// LOCAL_NAME? (anim ref)
    pub save_index: Option<i16>,
    /// FROM_NODE / FROM_NODE_POS / FROM_INPUT_NODE / FROM_INPUT_NODE_POS
    pub from_node: Option<CallObjectConnectorTarget>,
    /// TO_NODE / TO_NODE_POS / FROM_INPUT_NODE / TO_INPUT_NODE_POS
    pub to_node: Option<CallObjectConnectorTarget>,
    /// FROM_POS / FROM_INPUT_POS
    ///
    /// Warning: This can be ignored if `from_node` is set!
    pub from_pos: Option<ObjectConnectorPos>,
    /// TO_POS / TO_INPUT_POS
    ///
    /// Warning: This can be ignored if `to_node` is set!
    pub to_pos: Option<ObjectConnectorPos>,
}

/// CAMERA_STATE Index: 20
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CameraState {
    /// NAME (node name)
    pub name: String,
    /// NEAR_CLIP
    pub clip_near: Option<f32>,
    /// FAR_CLIP
    pub clip_far: Option<f32>,
    /// LOD_MULTIPLIER
    pub lod_multiplier: Option<f32>,
    /// H_FOV (not in reader)
    pub fov_h: Option<f32>,
    /// V_FOV (not in reader)
    pub fov_v: Option<f32>,
    /// H_ZOOM
    pub zoom_h: Option<f32>,
    /// V_ZOOM
    pub zoom_v: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FloatFromTo {
    pub from: f32,
    pub to: f32,
}

/// CAMERA_FROM_TO Index: 21
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CameraFromTo {
    /// NAME (node name)
    pub name: String,
    /// NEAR_CLIP_FROM_TO (not in reader)
    pub clip_near: Option<FloatFromTo>,
    /// FAR_CLIP_FROM_TO (not in reader)
    pub clip_far: Option<FloatFromTo>,
    /// LOD_MULTIPLIER_FROM_TO (not in reader)
    pub lod_multiplier: Option<FloatFromTo>,
    /// H_FOV_FROM_TO (not in reader)
    pub fov_h: Option<FloatFromTo>,
    /// V_FOV_FROM_TO (not in reader)
    pub fov_v: Option<FloatFromTo>,
    /// H_ZOOM_FROM_TO
    pub zoom_h: Option<FloatFromTo>,
    /// V_ZOOM_FROM_TO
    pub zoom_v: Option<FloatFromTo>,
    /// RUN_TIME
    pub run_time: f32,
}

/// CALL_SEQUENCE Index: 22
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallSequence {
    /// NAME (sequence name)
    pub name: String,
}

/// STOP_SEQUENCE Index: 23
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct StopSequence {
    /// NAME (sequence name)
    pub name: String,
}

/// AT_NODE
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallAnimationAtNode {
    /// node name or INPUT_NODE
    pub node: String,
    /// Warning: If and only if the node is "INPUT_NODE", the anim def's input
    /// position is added to the position value. Otherwise, only the position
    /// value is used; the node's position is not used.
    pub position: Option<Vec3>,
    /// Warning: If this is set, then the position and translate are somehow
    /// derived from the node, and finally this translate value is added to the
    /// node's translate value. I'm also unsure if the position value in this
    /// struct is used at all.
    pub translate: Option<Vec3>,
}

/// WITH_NODE
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallAnimationWithNode {
    /// node name or INPUT_NODE
    pub node: String,
    /// Warning: If and only if the node is "INPUT_NODE", the anim def's input
    /// position is added to the position value. Otherwise, only the position
    /// value is used; the node's position is not used.
    pub position: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum CallAnimationParameters {
    /// AT_NODE
    AtNode(CallAnimationAtNode),
    /// WITH_NODE
    WithNode(CallAnimationWithNode),
}

/// CALL_ANIMATION Index: 24
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct CallAnimation {
    /// NAME (anim name)
    pub name: String,
    /// OPERAND_NODE (node name)
    pub operand_node: Option<String>,
    /// WAIT_FOR_COMPLETION (anim ref)
    pub wait_for_completion: Option<i16>,
    /// AT_NODE / WITH_NODE
    pub parameters: Option<CallAnimationParameters>,
}

/// STOP_ANIMATION Index: 25
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct StopAnimation {
    /// NAME (anim name)
    pub name: String,
}

/// RESET_ANIMATION Index: 26
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ResetAnimation {
    /// NAME (anim name)
    pub name: String,
}

/// INVALIDATE_ANIMATION Index: 27
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct InvalidateAnimation {
    /// NAME (anim name)
    pub name: String,
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum FogType: u32 {
        /// OFF
        Off = 0,
        /// LINEAR
        Linear = 1,
        /// EXPONENTIAL (not in reader)
        Exponential = 2,
    }
}

/// FOG_STATE Index: 28
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FogState {
    /// TYPE
    pub type_: Option<FogType>,
    /// COLOR
    pub color: Option<Color>,
    /// ALTITUDE
    pub altitude: Option<Range>,
    /// RANGE
    pub range: Option<Range>,
}

// no index 29

/// LOOP Index 30
#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum Loop {
    /// LOOP_COUNT
    Count(i16),
    /// LOOP_RUN_TIME (not in reader)
    RunTime(f32),
}

/// NODE_UNDERCOVER
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Struct)]
#[dotnet(val_struct)]
pub struct NodeUndercover {
    pub node_index: u32,
    pub distance: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Union)]
pub enum Condition {
    /// RANDOM_WEIGHT
    RandomWeight(f32),
    /// PLAYER_RANGE (squared?)
    PlayerRange(f32),
    /// ANIMATION_LOD
    AnimationLod(u32),
    /// NODE_UNDERCOVER
    NodeUndercover(NodeUndercover),
    /// HW_RENDER
    HwRender(bool),
    /// PLAYER_1ST_PERSON
    PlayerFirstPerson(bool),
}

/// IF Index: 31
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct If {
    pub condition: Condition,
}

/// ELSE Index: 32
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Else {}

/// ELSEIF Index: 33
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Elseif {
    pub condition: Condition,
}

/// ENDIF Index: 34
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Endif {}

/// CALLBACK Index: 35
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct Callback {
    /// VALUE
    pub value: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
#[dotnet(val_struct)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// FBFX_COLOR_FROM_TO Index: 36
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FbfxColorFromTo {
    /// FROM
    pub from: Rgba,
    /// TO
    pub to: Rgba,
    /// RUN_TIME
    pub run_time: f32,
    /// Only used for binary accuracy.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub alpha_delta: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FbfxCsinwaveScreenPos {
    pub x: FloatFromTo,
    pub y: FloatFromTo,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FbfxCsinwaveCsin {
    pub x: FloatFromTo,
    pub y: FloatFromTo,
    pub z: FloatFromTo,
}

/// FBFX_CSINWAVE_FROM_TO Index: 37
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct FbfxCsinwaveFromTo {
    /// AT_NODE (node name)
    pub at_node: Option<AtNode>,
    /// SCREEN_POS_FROM / SCREEN_POS_TO
    pub screen_pos: Option<FbfxCsinwaveScreenPos>,
    /// WORLD_RADIUS_FROM / WORLD_RADIUS_TO
    pub world_radius: Option<FloatFromTo>,
    /// SCREEN_RADIUS_FROM / SCREEN_RADIUS_TO
    pub screen_radius: Option<FloatFromTo>,
    /// CSIN_FROM / CSIN_TO
    pub csin: FbfxCsinwaveCsin,
    /// RUN_TIME
    pub run_time: f32,
}

/// ANIM_VERBOSE Index: 39
///
/// This does nothing, even in RC.
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct AnimVerbose {
    /// ON
    pub on: bool,
}

/// DETONATE_WEAPON Index: 41
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct DetonateWeapon {
    /// WEAPON
    pub weapon: String,
    /// AT_NODE (node name or INPUT_NODE)
    pub at_node: AtNode,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Enum)]
pub enum PufferIntervalType {
    Time,
    Distance,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct PufferInterval {
    pub interval_type: PufferIntervalType,
    pub interval_value: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct PufferIntervalGarbage {
    pub interval_type: PufferIntervalType,
    pub has_interval_type: bool,
    pub interval_value: f32,
    pub has_interval_value: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct PufferStateTexture {
    pub name: String,
    pub run_time: Option<f32>,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Struct,
)]
#[repr(C)]
pub struct PufferStateColor {
    pub unk00: f32,
    pub color: Color,
    pub unk16: f32,
}
impl_as_bytes!(PufferStateColor, 20);

impl PufferStateColor {
    pub const ZERO: Self = Self {
        unk00: 0.0,
        color: Color::BLACK,
        unk16: 0.0,
    };
}

/// PUFFER_STATE Index: 42
#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct PufferState {
    /// NAME (puffer name)
    pub name: String,
    /// ACTIVE_STATE
    ///
    /// Warning: Not a boolean?
    pub active_state: Option<u32>,
    /// AT_NODE TODO
    pub translate: Option<Vec3>,
    /// AT_NODE TODO
    pub at_node: Option<String>,
    /// LOCAL_VELOCITY
    pub local_velocity: Option<Vec3>,
    /// WORLD_VELOCITY
    pub world_velocity: Option<Vec3>,
    /// MIN_RANDOM_VELOCITY
    pub min_random_velocity: Option<Vec3>,
    /// MAX_RANDOM_VELOCITY
    pub max_random_velocity: Option<Vec3>,
    /// WORLD_ACCELERATION
    pub world_acceleration: Option<Vec3>,
    /// DISTANCE_INTERVAL / TIME_INTERVAL
    pub interval: Option<PufferInterval>,
    /// SIZE_RANGE
    pub size_range: Option<Range>,
    /// LIFETIME_RANGE
    pub lifetime_range: Option<Range>,
    /// START_AGE_RANGE
    pub start_age_range: Option<Range>,
    /// DEVIATION_DISTANCE
    pub deviation_distance: Option<f32>,
    /// (not in reader)
    pub unk_range: Option<Range>,
    /// FADE_RANGE
    pub fade_range: Option<Range>,
    /// FRICTION
    pub friction: Option<f32>,
    /// WIND_FACTOR
    pub wind_factor: Option<f32>,
    /// PRIORITY
    pub priority: Option<f32>,
    /// NUMBER
    pub number: Option<u32>,
    /// TEXTURES
    pub textures: Option<Vec<PufferStateTexture>>,
    /// COLORS
    pub colors: Option<Vec<PufferStateColor>>,
    /// GROWTH_FACTOR
    pub growth_factors: Option<Vec<Range>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub interval_garbage: Option<PufferIntervalGarbage>,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum EventData {
    Sound(Sound),
    SoundNode(SoundNode),
    Effect(Effect),
    LightState(LightState),
    LightAnimation(LightAnimation),
    ObjectActiveState(ObjectActiveState),
    ObjectTranslateState(ObjectTranslateState),
    ObjectScaleState(ObjectScaleState),
    ObjectRotateState(ObjectRotateState),
    ObjectMotion(ObjectMotion),
    ObjectMotionFromTo(ObjectMotionFromTo),
    ObjectMotionSiScript(ObjectMotionSiScript),
    ObjectOpacityState(ObjectOpacityState),
    ObjectOpacityFromTo(ObjectOpacityFromTo),
    ObjectAddChild(ObjectAddChild),
    ObjectDeleteChild(ObjectDeleteChild),
    ObjectCycleTexture(ObjectCycleTexture),
    ObjectConnector(ObjectConnector),
    CallObjectConnector(CallObjectConnector),
    CameraState(CameraState),
    CameraFromTo(CameraFromTo),
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
    Elseif(Elseif),
    Endif(Endif),
    Callback(Callback),
    FbfxColorFromTo(FbfxColorFromTo),
    FbfxCsinwaveFromTo(FbfxCsinwaveFromTo),
    AnimVerbose(AnimVerbose),
    DetonateWeapon(DetonateWeapon),
    PufferState(PufferState),
}
