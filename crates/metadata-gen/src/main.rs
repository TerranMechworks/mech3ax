mod csharp;
mod python;
mod resolver;

use mech3ax_api_types as api;
use resolver::Resolver;

fn add_types(resolver: &mut impl Resolver) {
    // --- common.rs
    resolver.push::<api::AffineMatrix>();
    resolver.push::<api::Color>();
    resolver.push::<api::Matrix>();
    resolver.push::<api::Quaternion>();
    resolver.push::<api::Range>();
    resolver.push::<api::Vec3>();
    resolver.push::<api::Count>();
    resolver.push::<api::Index>();

    // --- zmap.rs
    resolver.push::<api::zmap::MapColor>();
    resolver.push::<api::zmap::MapFeature>();
    resolver.push::<api::zmap::Zmap>();

    // --- motion.rs
    resolver.push::<api::motion::MotionFrame>();
    resolver.push::<api::motion::MotionPart>();
    resolver.push::<api::motion::Motion>();

    // --- messages.rs
    resolver.push::<api::messages::MessageEntry>();
    resolver.push::<api::messages::Messages>();

    // --- interp.rs
    resolver.push::<api::interp::Script>();

    // --- image.rs
    resolver.push::<api::image::TextureAlpha>();
    resolver.push::<api::image::TextureStretch>();
    resolver.push::<api::image::PaletteData>();
    resolver.push::<api::image::GlobalPalette>();
    resolver.push::<api::image::TexturePalette>();
    resolver.push::<api::image::TextureInfo>();
    resolver.push::<api::image::TextureManifest>();

    // --- archive.rs
    resolver.push::<api::archive::ArchiveEntryInfoValid>();
    resolver.push::<api::archive::ArchiveEntryInfoInvalid>();
    resolver.push::<api::archive::ArchiveEntryInfo>();
    resolver.push::<api::archive::ArchiveEntry>();
}

fn add_gamez(resolver: &mut impl Resolver) {
    // --- gamez/materials.rs
    resolver.push::<api::gamez::materials::CycleData>();
    resolver.push::<api::gamez::materials::Soil>();
    resolver.push::<api::gamez::materials::TexturedMaterial>();
    resolver.push::<api::gamez::materials::ColoredMaterial>();
    resolver.push::<api::gamez::materials::Material>();

    // --- gamez/model.rs
    resolver.push::<api::gamez::model::UvCoord>();
    resolver.push::<api::gamez::model::PointLight>();
    resolver.push::<api::gamez::model::PolygonFlags>();
    resolver.push::<api::gamez::model::PolygonMaterial>();
    resolver.push::<api::gamez::model::Polygon>();
    resolver.push::<api::gamez::model::ModelType>();
    resolver.push::<api::gamez::model::FacadeMode>();
    resolver.push::<api::gamez::model::ModelFlags>();
    resolver.push::<api::gamez::model::Model>();

    // nodes required for mechlib
    add_nodes(resolver);

    // --- gamez/mod.rs
    resolver.push::<api::gamez::MechlibModel>();
    resolver.push::<api::gamez::Texture>();
    resolver.push::<api::gamez::GameZMetadata>();
    resolver.push::<api::gamez::GameZ>();
}

fn add_nodes(resolver: &mut impl Resolver) {
    // --- gamez/nodes.rs
    resolver.push::<api::gamez::nodes::NodeFlags>();
    resolver.push::<api::gamez::nodes::ActiveBoundingBox>();
    resolver.push::<api::gamez::nodes::BoundingBox>();
    resolver.push::<api::gamez::nodes::Partition>();
    resolver.push::<api::gamez::nodes::Display>();
    resolver.push::<api::gamez::nodes::Camera>();
    resolver.push::<api::gamez::nodes::LightFlags>();
    resolver.push::<api::gamez::nodes::Light>();
    resolver.push::<api::gamez::nodes::Lod>();
    resolver.push::<api::gamez::nodes::RotateTranslateScale>();
    resolver.push::<api::gamez::nodes::Transform>();
    resolver.push::<api::gamez::nodes::Object3d>();
    resolver.push::<api::gamez::nodes::Window>();
    resolver.push::<api::gamez::nodes::FogType>();
    resolver.push::<api::gamez::nodes::Area>();
    resolver.push::<api::gamez::nodes::WorldFog>();
    resolver.push::<api::gamez::nodes::WorldPartitionValue>();
    resolver.push::<api::gamez::nodes::WorldPartition>();
    resolver.push::<api::gamez::nodes::WorldPtrs>();
    resolver.push::<api::gamez::nodes::World>();
    resolver.push::<api::gamez::nodes::NodeData>();
    resolver.push::<api::gamez::nodes::Node>();
}

fn add_events(resolver: &mut impl Resolver) {
    // --- anim/events.rs

    // events
    resolver.push::<api::anim::events::AtNode>();
    resolver.push::<api::anim::events::Translate>();
    resolver.push::<api::anim::events::Vec3FromTo>();
    resolver.push::<api::anim::events::FloatFromTo>();

    // 01
    resolver.push::<api::anim::events::Sound>();
    // 02
    resolver.push::<api::anim::events::SoundNode>();
    // 03
    resolver.push::<api::anim::events::Effect>();
    // 04
    resolver.push::<api::anim::events::LightType>();
    resolver.push::<api::anim::events::LightState>();
    // 05
    resolver.push::<api::anim::events::LightAnimation>();
    // 06
    resolver.push::<api::anim::events::ObjectActiveState>();
    // 07
    resolver.push::<api::anim::events::ObjectTranslateState>();
    // 08
    resolver.push::<api::anim::events::ObjectScaleState>();
    // 09
    resolver.push::<api::anim::events::RotateBasis>();
    resolver.push::<api::anim::events::ObjectRotateState>();
    // 10
    resolver.push::<api::anim::events::Gravity>();
    resolver.push::<api::anim::events::TranslationRange>();
    resolver.push::<api::anim::events::ObjectMotionTranslation>();
    resolver.push::<api::anim::events::ForwardRotationTime>();
    resolver.push::<api::anim::events::ForwardRotationDistance>();
    resolver.push::<api::anim::events::ForwardRotation>();
    resolver.push::<api::anim::events::ObjectMotionXyzRot>();
    resolver.push::<api::anim::events::ObjectMotionScale>();
    resolver.push::<api::anim::events::BounceSequences>();
    resolver.push::<api::anim::events::BounceSound>();
    resolver.push::<api::anim::events::BounceSounds>();
    resolver.push::<api::anim::events::ObjectMotion>();
    // 11
    resolver.push::<api::anim::events::ObjectMotionFromTo>();
    // 12
    resolver.push::<api::anim::events::ObjectMotionSiScript>();
    // 13
    resolver.push::<api::anim::events::ObjectOpacityState>();
    // 14
    resolver.push::<api::anim::events::ObjectOpacity>();
    resolver.push::<api::anim::events::ObjectOpacityFromTo>();
    // 15
    resolver.push::<api::anim::events::ObjectAddChild>();
    // 16
    resolver.push::<api::anim::events::ObjectDeleteChild>();
    // 17
    resolver.push::<api::anim::events::ObjectCycleTexture>();
    // 18
    resolver.push::<api::anim::events::ObjectConnectorPos>();
    resolver.push::<api::anim::events::ObjectConnectorTime>();
    resolver.push::<api::anim::events::ObjectConnector>();
    // 19
    resolver.push::<api::anim::events::CallObjectConnectorTarget>();
    resolver.push::<api::anim::events::CallObjectConnector>();
    // 20
    resolver.push::<api::anim::events::CameraState>();
    // 21
    resolver.push::<api::anim::events::CameraFromTo>();
    // 22
    resolver.push::<api::anim::events::CallSequence>();
    // 23
    resolver.push::<api::anim::events::StopSequence>();
    // 24
    resolver.push::<api::anim::events::CallAnimationAtNode>();
    resolver.push::<api::anim::events::CallAnimationWithNode>();
    resolver.push::<api::anim::events::CallAnimationParameters>();
    resolver.push::<api::anim::events::CallAnimation>();
    // 25
    resolver.push::<api::anim::events::StopAnimation>();
    // 26
    resolver.push::<api::anim::events::ResetAnimation>();
    // 27
    resolver.push::<api::anim::events::InvalidateAnimation>();
    // 28
    resolver.push::<api::anim::events::FogType>();
    resolver.push::<api::anim::events::FogState>();
    // 29
    // 30
    resolver.push::<api::anim::events::Loop>();
    // 31-34
    resolver.push::<api::anim::events::NodeUndercover>();
    resolver.push::<api::anim::events::Condition>();
    resolver.push::<api::anim::events::If>();
    resolver.push::<api::anim::events::Else>();
    resolver.push::<api::anim::events::Elseif>();
    resolver.push::<api::anim::events::Endif>();
    // 35
    resolver.push::<api::anim::events::Callback>();
    // 36
    resolver.push::<api::anim::events::Rgba>();
    resolver.push::<api::anim::events::FbfxColorFromTo>();
    // 37
    resolver.push::<api::anim::events::FbfxCsinwaveScreenPos>();
    resolver.push::<api::anim::events::FbfxCsinwaveCsin>();
    resolver.push::<api::anim::events::FbfxCsinwaveFromTo>();
    // 38
    // 39
    resolver.push::<api::anim::events::AnimVerbose>();
    // 40
    // 41
    resolver.push::<api::anim::events::DetonateWeapon>();
    // 42
    resolver.push::<api::anim::events::PufferIntervalType>();
    resolver.push::<api::anim::events::PufferInterval>();
    resolver.push::<api::anim::events::PufferIntervalGarbage>();
    resolver.push::<api::anim::events::PufferStateTexture>();
    resolver.push::<api::anim::events::PufferStateColor>();
    resolver.push::<api::anim::events::PufferState>();

    // event
    resolver.push::<api::anim::events::StartOffset>();
    resolver.push::<api::anim::events::EventStart>();
    resolver.push::<api::anim::events::EventData>();
    resolver.push::<api::anim::events::Event>();
}

fn add_anim(resolver: &mut impl Resolver) {
    // --- anim/si_script.rs
    resolver.push::<api::anim::TranslateData>();
    resolver.push::<api::anim::RotateData>();
    resolver.push::<api::anim::ScaleData>();
    resolver.push::<api::anim::ObjectMotionSiFrame>();
    resolver.push::<api::anim::SiScript>();

    // --- anim/activation_prereq.rs
    resolver.push::<api::anim::PrerequisiteAnimation>();
    resolver.push::<api::anim::PrerequisiteObject>();
    resolver.push::<api::anim::PrerequisiteParent>();
    resolver.push::<api::anim::ActivationPrerequisite>();

    // --- anim/support.rs
    resolver.push::<api::anim::AnimRefCallAnimation>();
    resolver.push::<api::anim::AnimRefCallObjectConnector>();
    resolver.push::<api::anim::AnimRef>();
    resolver.push::<api::anim::ObjectRef>();
    resolver.push::<api::anim::NodeRef>();
    resolver.push::<api::anim::LightRef>();
    resolver.push::<api::anim::PufferRef>();
    resolver.push::<api::anim::DynamicSoundRef>();
    resolver.push::<api::anim::StaticSoundRef>();
    resolver.push::<api::anim::EffectRef>();

    // --- anim/anim_def.rs (part 1)
    resolver.push::<api::anim::AnimDefFile>();
    resolver.push::<api::anim::AnimActivation>();
    resolver.push::<api::anim::Execution>();
    resolver.push::<api::anim::NamePad>();
    resolver.push::<api::anim::NamePtrFlags>();
    resolver.push::<api::anim::SeqDefState>();

    add_events(resolver);

    // --- anim/anim_def.rs (part 2)
    resolver.push::<api::anim::ResetState>();
    resolver.push::<api::anim::SeqDef>();
    resolver.push::<api::anim::AnimDefPtrs>();
    resolver.push::<api::anim::AnimDef>();

    // --- anim/mod.rs
    resolver.push::<api::anim::AnimMission>();
    resolver.push::<api::anim::AnimMetadata>();
}

fn main() {
    csharp();
    python();
}

fn csharp() {
    let mut resolver = csharp::TypeResolver::new();

    add_types(&mut resolver);
    add_gamez(&mut resolver);
    add_anim(&mut resolver);

    csharp::write(resolver);
}

fn python() {
    let mut resolver = python::TypeResolver::new();

    add_types(&mut resolver);
    add_gamez(&mut resolver);
    add_anim(&mut resolver);

    python::write(resolver);
}
