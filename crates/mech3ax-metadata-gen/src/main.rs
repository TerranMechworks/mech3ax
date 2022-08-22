mod enums;
mod fields;
mod options;
mod resolver;
mod structs;
mod templates;
mod unions;

use mech3ax_api_types as api;
use resolver::TypeResolver;

fn main() {
    let mut resolver = TypeResolver::new();

    // types
    resolver.push_struct::<api::Range>();
    resolver.push_struct::<api::Vec3>();
    resolver.push_struct::<api::Color>();
    resolver.push_struct::<api::Quaternion>();
    resolver.push_struct::<api::Matrix>();
    // archive
    resolver.push_struct::<api::ArchiveEntry>();
    // image
    resolver.push_enum::<api::TextureAlpha>();
    resolver.push_enum::<api::TextureStretch>();
    resolver.push_struct::<api::PaletteData>();
    resolver.push_struct::<api::GlobalPalette>();
    resolver.push_union::<api::TexturePalette>();
    resolver.push_struct::<api::TextureInfo>();
    resolver.push_struct::<api::TextureManifest>();
    // interp
    resolver.push_struct::<api::Script>();
    // messages
    resolver.push_struct::<api::MessageEntry>();
    resolver.push_struct::<api::Messages>();
    // motion
    resolver.push_struct::<api::MotionFrame>();
    resolver.push_struct::<api::MotionPart>();
    resolver.push_struct::<api::Motion>();
    // gamez materials
    resolver.push_struct::<api::CycleData>();
    resolver.push_struct::<api::TexturedMaterial>();
    resolver.push_struct::<api::ColoredMaterial>();
    resolver.push_union::<api::Material>();
    // gamez mesh
    resolver.push_struct::<api::UvCoord>();
    resolver.push_struct::<api::Polygon>();
    resolver.push_struct::<api::MeshLight>();
    resolver.push_struct::<api::Mesh>();
    // gamez nodes
    resolver.push_struct::<api::AreaPartition>();
    resolver.push_struct::<api::Area>();
    resolver.push_struct::<api::BoundingBox>();
    resolver.push_struct::<api::Transformation>();
    resolver.push_struct::<api::Partition>();
    resolver.push_struct::<api::NodeFlags>();
    resolver.push_struct::<api::Camera>();
    resolver.push_struct::<api::Display>();
    resolver.push_struct::<api::Empty>();
    resolver.push_struct::<api::Light>();
    resolver.push_struct::<api::Lod>();
    resolver.push_struct::<api::Object3d>();
    resolver.push_struct::<api::Window>();
    resolver.push_struct::<api::World>();
    resolver.push_union::<api::Node>();
    // gamez mechlib
    resolver.push_struct::<api::Model>();
    // gamez mod
    resolver.push_struct::<api::GameZMetadata>();
    resolver.push_struct::<api::GameZData>();
    // anim mod
    resolver.push_struct::<api::AnimName>();
    resolver.push_struct::<api::AnimPtr>();
    resolver.push_struct::<api::AnimMetadata>();
    resolver.push_enum::<api::AnimActivation>();
    resolver.push_union::<api::Execution>();
    resolver.push_struct::<api::NamePad>();
    resolver.push_struct::<api::NamePtr>();
    resolver.push_struct::<api::NamePtrFlags>();
    resolver.push_enum::<api::SeqActivation>();
    resolver.push_struct::<api::PrereqAnimation>();
    resolver.push_struct::<api::PrereqObject>();
    resolver.push_union::<api::ActivationPrereq>();
    // anim events
    resolver.push_struct::<api::AtNode>();
    resolver.push_struct::<api::StopAnimation>();
    resolver.push_struct::<api::ResetAnimation>();
    resolver.push_struct::<api::InvalidateAnimation>();
    resolver.push_struct::<api::CallAnimationAtNode>();
    resolver.push_struct::<api::CallAnimationWithNode>();
    resolver.push_struct::<api::CallAnimationTargetNode>();
    resolver.push_union::<api::CallAnimationParameters>();
    resolver.push_struct::<api::CallAnimation>();
    resolver.push_struct::<api::CallObjectConnector>();
    resolver.push_struct::<api::Loop>();
    resolver.push_struct::<api::RandomWeightCond>();
    resolver.push_struct::<api::PlayerRangeCond>();
    resolver.push_struct::<api::AnimationLodCond>();
    resolver.push_struct::<api::HwRenderCond>();
    resolver.push_struct::<api::PlayerFirstPersonCond>();
    resolver.push_union::<api::If>();
    resolver.push_union::<api::ElseIf>();
    resolver.push_struct::<api::Else>();
    resolver.push_struct::<api::EndIf>();
    resolver.push_struct::<api::Callback>();
    resolver.push_struct::<api::DetonateWeapon>();
    resolver.push_struct::<api::Rgba>();
    resolver.push_struct::<api::FrameBufferEffectColor>();
    resolver.push_enum::<api::FogType>();
    resolver.push_struct::<api::FogState>();
    resolver.push_struct::<api::LightAnimation>();
    resolver.push_struct::<api::LightState>();
    resolver.push_struct::<api::ObjectActiveState>();
    resolver.push_struct::<api::ObjectAddChild>();
    resolver.push_struct::<api::ObjectConnector>();
    resolver.push_struct::<api::ObjectCycleTexture>();
    resolver.push_struct::<api::FloatFromTo>();
    resolver.push_struct::<api::Vec3FromTo>();
    resolver.push_struct::<api::ObjectMotionFromTo>();
    resolver.push_struct::<api::TranslateData>();
    resolver.push_struct::<api::RotateData>();
    resolver.push_struct::<api::ScaleData>();
    resolver.push_struct::<api::ObjectMotionSiFrame>();
    resolver.push_struct::<api::ObjectMotionSiScript>();
    resolver.push_enum::<api::GravityMode>();
    resolver.push_struct::<api::Gravity>();
    resolver.push_struct::<api::ForwardRotationTime>();
    resolver.push_struct::<api::ForwardRotationDistance>();
    resolver.push_union::<api::ForwardRotation>();
    resolver.push_struct::<api::XyzRotation>();
    resolver.push_struct::<api::ObjectMotionTranslation>();
    resolver.push_struct::<api::ObjectMotionScale>();
    resolver.push_struct::<api::BounceSequence>();
    resolver.push_struct::<api::BounceSound>();
    resolver.push_struct::<api::ObjectMotion>();
    resolver.push_struct::<api::ObjectOpacity>();
    resolver.push_struct::<api::ObjectOpacityFromTo>();
    resolver.push_struct::<api::ObjectOpacityState>();
    resolver.push_union::<api::RotateState>();
    resolver.push_struct::<api::ObjectRotateState>();
    resolver.push_struct::<api::ObjectScaleState>();
    resolver.push_struct::<api::ObjectTranslateState>();
    resolver.push_enum::<api::IntervalType>();
    resolver.push_struct::<api::Interval>();
    resolver.push_struct::<api::PufferStateCycleTextures>();
    resolver.push_struct::<api::PufferState>();
    resolver.push_struct::<api::CallSequence>();
    resolver.push_struct::<api::StopSequence>();
    resolver.push_struct::<api::SoundNode>();
    resolver.push_struct::<api::Sound>();
    // resolver.push_union::<api::EventData>();
    resolver.push_enum::<api::StartOffset>();
    resolver.push_struct::<api::EventStart>();
    // resolver.push_struct::<api::Event>();
    // anim mod
    // resolver.push_struct::<api::SeqDef>();
    // resolver.push_struct::<api::AnimDef>();

    let tera = templates::make_tera();
    let (enums, structs, unions, options) = resolver.into_values();

    for item in enums {
        let contents = item.render_impl(&tera).unwrap();
        let path = format!("output/Impl/{}.cs", item.name);
        std::fs::write(path, contents).unwrap();
        let contents = item.render_conv(&tera).unwrap();
        let path = format!("output/Conv/{}Converter.cs", item.name);
        std::fs::write(path, contents).unwrap();
    }

    for item in structs {
        let contents = item.render_impl(&tera).unwrap();
        let path = format!("output/Impl/{}.cs", item.name);
        std::fs::write(path, contents).unwrap();
        let contents = item.render_conv(&tera).unwrap();
        let path = format!("output/Conv/{}Converter.cs", item.name);
        std::fs::write(path, contents).unwrap();
    }

    for item in unions {
        let contents = item.render_impl(&tera).unwrap();
        let path = format!("output/Impl/{}.cs", item.name);
        std::fs::write(path, contents).unwrap();
        let contents = item.render_conv(&tera).unwrap();
        let path = format!("output/Conv/{}Converter.cs", item.name);
        std::fs::write(path, contents).unwrap();
    }

    let contents = options.render_impl(&tera).unwrap();
    let path = format!("output/Conv/{}.cs", "Options");
    std::fs::write(path, contents).unwrap();

    let factories = options.into_factories();
    for item in factories {
        let contents = item.render_impl(&tera).unwrap();
        let path = format!("output/Conv/{}ConverterFactory.cs", item.name);
        std::fs::write(path, contents).unwrap();
    }
}
