use crate::{api, num, sum, Color, Range, Vec3};
use mech3ax_types::impl_as_bytes;

num! {
    enum StartOffset: u8 {
        Animation = 1,
        Sequence = 2,
        Event = 3,
    }
}

api! {
    struct EventStart : Val {
        offset: StartOffset,
        time: f32,
    }
}

api! {
    struct Event {
        start: Option<EventStart>,
        data: EventData,
    }
}

api! {
    /// AT_NODE
    struct AtNode : Val {
        /// node name
        ///
        /// Warning: Whether INPUT_NODE is allowed depends on the parent struct!
        name: String,
        pos: Vec3,
    }
}

sum! {
    enum Translate {
        Absolute(Vec3),
        AtNode(AtNode),
    }
}

api! {
    /// SOUND Index: 01
    struct Sound {
        /// NAME (static sound name)
        name: String,
        /// AT_NODE (node name)
        at_node: Option<AtNode>,
    }
}

api! {
    /// SOUND_NODE Index: 02
    struct SoundNode {
        /// NAME (sound node name)
        name: String,
        /// ACTIVE_STATE
        active_state: bool,
        /// AT_NODE TODO (node name)
        translate: Option<Translate>,
    }
}

api! {
    /// EFFECT Index: 03
    struct Effect {
        /// NAME (effect name)
        name: String,
        /// AT_NODE (node name or INPUT_NODE)
        at_node: AtNode,
    }
}

num! {
    enum LightType: u32 {
        Directed = 0,
        PointSource = 1,
    }
}

api! {
    /// LIGHT_STATE Index: 04
    struct LightState {
        /// NAME (light name)
        name: String,
        /// ACTIVE_STATE
        active_state: bool,
        /// TYPE
        type_: LightType,
        /// AT_NODE TODO (node name or INPUT_NODE)
        translate: Option<Translate>,
        /// DIRECTIONAL
        directional: Option<bool>,
        /// SATURATED
        saturated: Option<bool>,
        /// SUBDIVIDE
        subdivide: Option<bool>,
        /// LIGHTMAP
        lightmap: Option<bool>,
        /// STATIC
        static_: Option<bool>,
        /// BICOLORED (not in reader)
        bicolored: Option<bool>,
        /// ORIENTATION (not in reader)
        orientation: Option<Vec3>,
        /// RANGE
        range: Option<Range>,
        /// COLOR
        color: Option<Color>,
        /// AMBIENT_COLOR (not in reader)
        ambient_color: Option<Color>,
        /// AMBIENT
        ambient: Option<f32>,
        /// DIFFUSE
        diffuse: Option<f32>,
    }
}

api! {
    /// LIGHT_ANIMATION Index: 05
    struct LightAnimation {
        /// NAME (light name)
        name: String,
        /// RANGE
        range: Range,
        /// COLOR
        color: Color,
        /// RUN_TIME
        run_time: f32,
        /// RANGE (second set of values?)
        range_alt: Option<Range> = { None },
    }
}

api! {
    /// OBJECT_ACTIVE_STATE Index: 06
    struct ObjectActiveState {
        /// NAME (node name or INPUT_NODE)
        node: String,
        /// STATE
        state: bool,
    }
}

api! {
    /// OBJECT_TRANSLATE_STATE Index: 07
    struct ObjectTranslateState {
        /// NAME (node name)
        node: String,
        /// STATE / RELATIVE
        relative: bool,
        /// STATE / RELATIVE
        state: Vec3,
        /// AT_NODE (node name or INPUT_NODE)
        at_node: Option<String>,
    }
}

api! {
    /// OBJECT_SCALE_STATE Index: 08
    struct ObjectScaleState {
        /// NAME (node name)
        name: String,
        /// STATE
        state: Vec3,
    }
}

sum! {
    enum RotateBasis {
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
}

api! {
    /// OBJECT_ROTATE_STATE Index: 09
    /// Camera and Object3d nodes only!
    struct ObjectRotateState {
        /// NAME (node name)
        name: String,
        /// STATE / AT_NODE_MATRIX / AT_NODE_XYZ (Radians)
        state: Vec3,
        /// STATE / AT_NODE_MATRIX / AT_NODE_XYZ
        basis: RotateBasis,
    }
}

api! {
    /// GRAVITY
    struct Gravity : Val {
        /// DEFAULT = -9.8
        value: f32,
        /// LOCAL or COMPLEX
        complex: bool,
        /// NO_ALTITUDE
        no_altitude: bool,
    }
}

api! {
    /// TRANSLATION_RANGE_MIN and TRANSLATION_RANGE_MAX
    struct TranslationRange {
        /// Radians
        xz: Range,
        y: Range,
        initial: Range,
        delta: Range,
    }
}

api! {
    /// TRANSLATION (unclear)
    struct ObjectMotionTranslation {
        initial: Vec3,
        delta: Vec3,
        rnd_xz: Vec3,
    }
}

api! {
    /// FORWARD_ROTATION TIME (`["TIME", <initial>, <delta>]`)
    struct ForwardRotationTime : Val {
        // Radians
        initial: f32,
        // Radians
        delta: f32,
    }
}

api! {
    /// FORWARD_ROTATION DIST (`["DISTANCE", <initial>, <delta_ign>]`)
    struct ForwardRotationDistance : Val {
        initial: f32,
    }
}

sum! {
    /// FORWARD_ROTATION
    enum ForwardRotation {
        /// TIME
        Time(ForwardRotationTime),
        /// DISTANCE
        Distance(ForwardRotationDistance),
    }
}

api! {
    /// XYZ_ROTATION
    struct ObjectMotionXyzRot {
        initial: Vec3,
        delta: Vec3,
    }
}

api! {
    /// SCALE
    struct ObjectMotionScale {
        initial: Vec3,
        delta: Vec3,
    }
}

api! {
    /// BOUNCE_SEQUENCE, BOUNCE_SEQUENCE_WATER, BOUNCE_SEQUENCE_LAVA
    struct BounceSequences {
        /// BOUNCE_SEQUENCE
        default: Option<String>,
        /// BOUNCE_SEQUENCE_WATER (not RC)
        water: Option<String>,
        /// BOUNCE_SEQUENCE_LAVA (not RC)
        lava: Option<String>,
    }
}

api! {
    /// BOUNCE_SOUND
    struct BounceSound {
        /// NAME
        name: String,
        /// FULL_VOLUME_VELOCITY
        volume: f32,
    }
}

api! {
    /// BOUNCE_SOUND, BOUNCE_SOUND_WATER, BOUNCE_SOUND_LAVA
    struct BounceSounds {
        /// BOUNCE_SOUND
        default: Option<BounceSound>,
        /// BOUNCE_SOUND_WATER (not in reader, not RC)
        water: Option<BounceSound>,
        /// BOUNCE_SOUND_LAVA (not in reader, not RC)
        lava: Option<BounceSound>,
    }
}

api! {
    /// OBJECT_MOTION Index: 10
    struct ObjectMotion {
        /// NAME (node name)
        node: String,
        /// IMPACT_FORCE
        impact_force: bool,
        /// MORPH (not in reader)
        morph: Option<f32>,
        /// GRAVITY
        gravity: Option<Gravity>,
        /// TRANSLATION_RANGE
        translation_range: Option<TranslationRange>,
        /// TRANSLATION
        translation: Option<ObjectMotionTranslation>,
        /// FORWARD_ROTATION
        forward_rotation: Option<ForwardRotation>,
        /// XYZ_ROTATION
        xyz_rotation: Option<ObjectMotionXyzRot>,
        /// SCALE
        scale: Option<ObjectMotionScale>,
        /// BOUNCE_SEQUENCE, BOUNCE_SEQUENCE_WATER, BOUNCE_SEQUENCE_LAVA
        bounce_sequence: Option<BounceSequences>,
        /// BOUNCE_SOUND
        bounce_sound: Option<BounceSounds>,
        /// RUN_TIME
        run_time: Option<f32>,
    }
}

api! {
    struct Vec3FromTo {
        from: Vec3,
        to: Vec3,
    }
}

api! {
    /// OBJECT_MOTION_FROM_TO Index: 11
    struct ObjectMotionFromTo {
        /// NAME (node name)
        name: String,
        /// RUN_TIME
        run_time: f32,
        /// MORPH_FROM / MORPH_TO
        morph: Option<FloatFromTo>,
        /// TRANSLATE_FROM / TRANSLATE_TO
        ///
        /// Warning: Only applies to Camera/Object3D nodes!
        translate: Option<Vec3FromTo>,
        /// ROTATE_FROM / ROTATE_TO
        ///
        /// Warning: Only applies to Camera/Object3D nodes!
        rotate: Option<Vec3FromTo>,
        /// SCALE_FROM / SCALE_TO
        ///
        /// Warning: Only applies to Object3D nodes!
        scale: Option<Vec3FromTo>,
        /// Only used for binary accuracy.
        translate_delta: Option<Vec3> = { None },
        /// Only used for binary accuracy.
        rotate_delta: Option<Vec3> = { None },
        /// Only used for binary accuracy.
        scale_delta: Option<Vec3> = { None },
    }
}

api! {
    /// OBJECT_MOTION_SI_SCRIPT Index: 12
    struct ObjectMotionSiScript {
        /// NAME (node name)
        name: String,
        /// SCRIPT_FILENAME
        index: u32,
    }
}

api! {
    /// OBJECT_OPACITY_STATE Index: 13
    struct ObjectOpacityState {
        /// NAME (node name or INPUT_NODE)
        name: String,
        /// STATE / IsSet in interp
        state: bool,
        /// STATE
        opacity: Option<f32>,
    }
}

api! {
    struct ObjectOpacity : Val {
        /// STATE
        opacity: f32,
        /// STATE / IsSet in interp
        state: Option<bool>,
    }
}

api! {
    /// OBJECT_OPACITY_FROM_TO Index: 14
    struct ObjectOpacityFromTo {
        /// NAME (node name)
        name: String,
        /// OPACITY_FROM
        opacity_from: ObjectOpacity,
        /// OPACITY_TO
        opacity_to: ObjectOpacity,
        /// RUN_TIME
        run_time: f32,
        /// Only used for binary accuracy.
        opacity_delta: Option<f32> = { None },
    }
}

api! {
    /// OBJECT_ADD_CHILD Index: 15
    struct ObjectAddChild {
        /// PARENT_CHILD (node name)
        parent: String,
        /// PARENT_CHILD (node name)
        child: String,
    }
}

api! {
    /// OBJECT_DELETE_CHILD Index: 16
    struct ObjectDeleteChild {
        /// PARENT_CHILD (node name)
        parent: String,
        /// PARENT_CHILD (node name)
        child: String,
    }
}

api! {
    /// OBJECT_CYCLE_TEXTURE Index: 17
    struct ObjectCycleTexture {
        /// NAME (node name)
        name: String,
        /// RESET (0..6)
        reset: u16,
    }
}

sum! {
    enum ObjectConnectorPos {
        /// FROM_POS / TO_POS
        Pos(Vec3),
        /// FROM_INPUT_POS / TO_INPUT_POS
        Input,
    }
}

sum! {
    enum ObjectConnectorTime {
        /// FROM_T / TO_T
        Scalar(f32),
        /// FROM_T_START + FROM_T_END / TO_T_START + FROM_T_END
        Range(Range),
    }
}

api! {
    /// OBJECT_CONNECTOR Index: 18
    struct ObjectConnector {
        /// NAME
        name: String,
        /// FROM_NODE / FROM_INPUT_NODE (node name or INPUT_NODE)
        from_node: Option<String>,
        /// TO_NODE / TO_INPUT_NODE (node name or INPUT_NODE)
        to_node: Option<String>,
        /// FROM_POS / FROM_INPUT_POS
        ///
        /// Warning: This is ignored if `from_node` is set!
        from_pos: Option<ObjectConnectorPos>,
        /// TO_POS / TO_INPUT_POS
        ///
        /// Warning: This is ignored if `to_node` is set!
        to_pos: Option<ObjectConnectorPos>,
        /// FROM_T_START + FROM_T_END / FROM_T
        from_t: Option<ObjectConnectorTime>,
        /// TO_T_START + FROM_T_END / TO_T
        to_t: Option<ObjectConnectorTime>,
        /// RUN_TIME
        run_time: f32,
        /// MAX_LENGTH
        max_length: Option<f32>,
    }
}

api! {
    struct CallObjectConnectorTarget {
        /// FROM_NODE / TO_NODE (node name or INPUT_NODE)
        name: String,
        /// FROM_NODE_POS / TO_NODE_POS (node name or INPUT_NODE)
        ///
        /// Warning: This overrides the position and unsets the node!
        pos: bool,
    }
}

api! {
    /// CALL_OBJECT_CONNECTOR Index: 19
    struct CallObjectConnector {
        /// NAME (anim name)
        name: String,
        /// LOCAL_NAME? (anim ref)
        save_index: Option<i16>,
        /// FROM_NODE / FROM_NODE_POS / FROM_INPUT_NODE / FROM_INPUT_NODE_POS
        from_node: Option<CallObjectConnectorTarget>,
        /// TO_NODE / TO_NODE_POS / FROM_INPUT_NODE / TO_INPUT_NODE_POS
        to_node: Option<CallObjectConnectorTarget>,
        /// FROM_POS / FROM_INPUT_POS
        ///
        /// Warning: This can be ignored if `from_node` is set!
        from_pos: Option<ObjectConnectorPos>,
        /// TO_POS / TO_INPUT_POS
        ///
        /// Warning: This can be ignored if `to_node` is set!
        to_pos: Option<ObjectConnectorPos>,
    }
}

api! {
    /// CAMERA_STATE Index: 20
    struct CameraState {
        /// NAME (node name)
        name: String,
        /// NEAR_CLIP
        clip_near: Option<f32>,
        /// FAR_CLIP
        clip_far: Option<f32>,
        /// LOD_MULTIPLIER
        lod_multiplier: Option<f32>,
        /// H_FOV (not in reader)
        fov_h: Option<f32>,
        /// V_FOV (not in reader)
        fov_v: Option<f32>,
        /// H_ZOOM
        zoom_h: Option<f32>,
        /// V_ZOOM
        zoom_v: Option<f32>,
    }
}

api! {
    struct FloatFromTo {
        from: f32,
        to: f32,
    }
}

api! {
    /// CAMERA_FROM_TO Index: 21
    struct CameraFromTo {
        /// NAME (node name)
        name: String,
        /// NEAR_CLIP_FROM_TO (not in reader)
        clip_near: Option<FloatFromTo>,
        /// FAR_CLIP_FROM_TO (not in reader)
        clip_far: Option<FloatFromTo>,
        /// LOD_MULTIPLIER_FROM_TO (not in reader)
        lod_multiplier: Option<FloatFromTo>,
        /// H_FOV_FROM_TO (not in reader)
        fov_h: Option<FloatFromTo>,
        /// V_FOV_FROM_TO (not in reader)
        fov_v: Option<FloatFromTo>,
        /// H_ZOOM_FROM_TO
        zoom_h: Option<FloatFromTo>,
        /// V_ZOOM_FROM_TO
        zoom_v: Option<FloatFromTo>,
        /// RUN_TIME
        run_time: f32,
    }
}

api! {
    /// CALL_SEQUENCE Index: 22
    struct CallSequence {
        /// NAME (sequence name)
        name: String,
    }
}

api! {
    /// STOP_SEQUENCE Index: 23
    struct StopSequence {
        /// NAME (sequence name)
        name: String,
    }
}

api! {
    /// AT_NODE
    struct CallAnimationAtNode {
        /// node name or INPUT_NODE
        node: String,
        /// Warning: If and only if the node is "INPUT_NODE", the anim def's input
        /// position is added to the position value. Otherwise, only the position
        /// value is used; the node's position is not used.
        position: Option<Vec3>,
        /// Warning: If this is set, then the position and translate are somehow
        /// derived from the node, and finally this translate value is added to the
        /// node's translate value. I'm also unsure if the position value in this
        /// struct is used at all.
        translate: Option<Vec3>,
    }
}

api! {
    /// WITH_NODE
    struct CallAnimationWithNode {
        /// node name or INPUT_NODE
        node: String,
        /// Warning: If and only if the node is "INPUT_NODE", the anim def's input
        /// position is added to the position value. Otherwise, only the position
        /// value is used; the node's position is not used.
        position: Option<Vec3>,
    }
}

sum! {
    enum CallAnimationParameters {
        /// AT_NODE
        AtNode(CallAnimationAtNode),
        /// WITH_NODE
        WithNode(CallAnimationWithNode),
    }
}

api! {
    /// CALL_ANIMATION Index: 24
    struct CallAnimation {
        /// NAME (anim name)
        name: String,
        /// OPERAND_NODE (node name)
        operand_node: Option<String>,
        /// WAIT_FOR_COMPLETION (anim ref)
        wait_for_completion: Option<i16>,
        /// AT_NODE / WITH_NODE
        parameters: Option<CallAnimationParameters>,
    }
}

api! {
    /// STOP_ANIMATION Index: 25
    struct StopAnimation {
        /// NAME (anim name)
        name: String,
    }
}

api! {
    /// RESET_ANIMATION Index: 26
    struct ResetAnimation {
        /// NAME (anim name)
        name: String,
    }
}

api! {
    /// INVALIDATE_ANIMATION Index: 27
    struct InvalidateAnimation {
        /// NAME (anim name)
        name: String,
    }
}

num! {
    enum FogType: u32 {
        /// OFF
        Off = 0,
        /// LINEAR
        Linear = 1,
        /// EXPONENTIAL (not in reader)
        Exponential = 2,
    }
}

api! {
    /// FOG_STATE Index: 28
    struct FogState {
        /// TYPE
        type_: Option<FogType>,
        /// COLOR
        color: Option<Color>,
        /// ALTITUDE
        altitude: Option<Range>,
        /// RANGE
        range: Option<Range>,
    }
}

// no index 29

sum! {
    /// LOOP Index 30
    enum Loop {
        /// LOOP_COUNT
        Count(i16),
        /// LOOP_RUN_TIME (not in reader)
        RunTime(f32),
    }
}

api! {
    /// NODE_UNDERCOVER
    struct NodeUndercover : Val {
        node_index: u32,
        distance: u32,
    }
}

sum! {
    enum Condition {
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
}

api! {
    /// IF Index: 31
    struct If {
        condition: Condition,
    }
}

api! {
    /// ELSE Index: 32
    struct Else {}
}

api! {
    /// ELSEIF Index: 33
    struct Elseif {
        condition: Condition,
    }
}

api! {
    /// ENDIF Index: 34
    struct Endif {}
}

api! {
    /// CALLBACK Index: 35
    struct Callback {
        /// VALUE
        value: u32,
    }
}

api! {
    struct Rgba : Val {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }
}

api! {
    /// FBFX_COLOR_FROM_TO Index: 36
    struct FbfxColorFromTo {
        /// FROM
        from: Rgba,
        /// TO
        to: Rgba,
        /// RUN_TIME
        run_time: f32,
        /// Only used for binary accuracy.
        alpha_delta: Option<f32> = { None },
    }
}

api! {
    struct FbfxCsinwaveScreenPos {
        x: FloatFromTo,
        y: FloatFromTo,
    }
}

api! {
    struct FbfxCsinwaveCsin {
        x: FloatFromTo,
        y: FloatFromTo,
        z: FloatFromTo,
    }
}

api! {
    /// FBFX_CSINWAVE_FROM_TO Index: 37
    struct FbfxCsinwaveFromTo {
        /// AT_NODE (node name)
        at_node: Option<AtNode>,
        /// SCREEN_POS_FROM / SCREEN_POS_TO
        screen_pos: Option<FbfxCsinwaveScreenPos>,
        /// WORLD_RADIUS_FROM / WORLD_RADIUS_TO
        world_radius: Option<FloatFromTo>,
        /// SCREEN_RADIUS_FROM / SCREEN_RADIUS_TO
        screen_radius: Option<FloatFromTo>,
        /// CSIN_FROM / CSIN_TO
        csin: FbfxCsinwaveCsin,
        /// RUN_TIME
        run_time: f32,
    }
}

api! {
    /// ANIM_VERBOSE Index: 39
    ///
    /// This does nothing, even in RC.
    struct AnimVerbose {
        /// ON
        on: bool,
    }
}

api! {
    /// DETONATE_WEAPON Index: 41
    struct DetonateWeapon {
        /// WEAPON
        weapon: String,
        /// AT_NODE (node name or INPUT_NODE)
        at_node: AtNode,
    }
}

num! {
    enum PufferIntervalType {
        Time = 1,
        Distance = 2,
    }
}

api! {
    struct PufferInterval {
        interval_type: PufferIntervalType,
        interval_value: f32,
    }
}

api! {
    struct PufferIntervalGarbage {
        interval_type: PufferIntervalType,
        has_interval_type: bool,
        interval_value: f32,
        has_interval_value: bool,
    }
}

api! {
    struct PufferStateTexture {
        name: String,
        run_time: Option<f32>,
    }
}

api! {
    #[repr(C)]
    struct PufferStateColor {
        unk00: f32,
        color: Color,
        unk16: f32,
    }
}
impl_as_bytes!(PufferStateColor, 20);

impl PufferStateColor {
    pub const ZERO: Self = Self {
        unk00: 0.0,
        color: Color::BLACK,
        unk16: 0.0,
    };
}

api! {
    /// PUFFER_STATE Index: 42
    struct PufferState {
        /// NAME (puffer name)
        name: String,
        /// ACTIVE_STATE
        ///
        /// Warning: Not a boolean?
        active_state: Option<u32>,
        /// AT_NODE TODO
        translate: Option<Vec3>,
        /// AT_NODE TODO
        at_node: Option<String>,
        /// LOCAL_VELOCITY
        local_velocity: Option<Vec3>,
        /// WORLD_VELOCITY
        world_velocity: Option<Vec3>,
        /// MIN_RANDOM_VELOCITY
        min_random_velocity: Option<Vec3>,
        /// MAX_RANDOM_VELOCITY
        max_random_velocity: Option<Vec3>,
        /// WORLD_ACCELERATION
        world_acceleration: Option<Vec3>,
        /// DISTANCE_INTERVAL / TIME_INTERVAL
        interval: Option<PufferInterval>,
        /// SIZE_RANGE
        size_range: Option<Range>,
        /// LIFETIME_RANGE
        lifetime_range: Option<Range>,
        /// START_AGE_RANGE
        start_age_range: Option<Range>,
        /// DEVIATION_DISTANCE
        deviation_distance: Option<f32>,
        /// (not in reader)
        unk_range: Option<Range>,
        /// FADE_RANGE
        fade_range: Option<Range>,
        /// FRICTION
        friction: Option<f32>,
        /// WIND_FACTOR
        wind_factor: Option<f32>,
        /// PRIORITY
        priority: Option<f32>,
        /// NUMBER
        number: Option<u32>,
        /// TEXTURES
        textures: Option<Vec<PufferStateTexture>>,
        /// COLORS
        colors: Option<Vec<PufferStateColor>>,
        /// GROWTH_FACTOR
        growth_factors: Option<Vec<Range>>,
        interval_garbage: Option<PufferIntervalGarbage> = { None },
    }
}

sum! {
    enum EventData {
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
}
