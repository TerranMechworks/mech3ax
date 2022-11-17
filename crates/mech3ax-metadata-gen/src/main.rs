mod csharp_type;
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
    resolver.push::<api::Range>();
    resolver.push::<api::Vec3>();
    resolver.push::<api::Color>();
    resolver.push::<api::Quaternion>();
    resolver.push::<api::Matrix>();

    // archive
    resolver.push::<api::ArchiveEntry>();

    // image
    resolver.push::<api::TextureAlpha>();
    resolver.push::<api::TextureStretch>();
    resolver.push::<api::PaletteData>();
    resolver.push::<api::GlobalPalette>();
    resolver.push::<api::TexturePalette>();
    resolver.push::<api::TextureInfo>();
    resolver.push::<api::TextureManifest>();

    // interp
    resolver.push::<api::Script>();

    // messages
    resolver.push::<api::MessageEntry>();
    resolver.push::<api::Messages>();

    // motion
    resolver.push::<api::MotionFrame>();
    resolver.push::<api::MotionPart>();
    resolver.push::<api::Motion>();

    // zmap
    resolver.push::<api::zmap::MapColor>();
    resolver.push::<api::zmap::MapVertex>();
    resolver.push::<api::zmap::MapFeature>();
    resolver.push::<api::zmap::MapRc>();

    // gamez materials
    resolver.push::<api::ColoredMaterial>();
    resolver.push::<api::CycleData>();
    resolver.push::<api::TexturedMaterial>();
    resolver.push::<api::Material>();

    // gamez mesh
    resolver.push::<api::UvCoord>();
    resolver.push::<api::MeshLight>();
    resolver.push::<api::PolygonMw>();
    resolver.push::<api::MeshMw>();
    resolver.push::<api::PolygonFlags>();
    resolver.push::<api::MeshTexture>();
    resolver.push::<api::PolygonTextureNg>();
    resolver.push::<api::PolygonNg>();
    resolver.push::<api::MeshNg>();
    resolver.push::<api::PolygonRc>();
    resolver.push::<api::MeshRc>();

    // gamez nodes
    resolver.push::<api::AreaPartition>();
    resolver.push::<api::Area>();
    resolver.push::<api::BoundingBox>();
    resolver.push::<api::Transformation>();
    resolver.push::<api::Partition>();
    resolver.push::<api::NodeFlags>();

    // gamez nodes mw
    resolver.push::<api::Camera>();
    resolver.push::<api::Display>();
    resolver.push::<api::Empty>();
    resolver.push::<api::Light>();
    resolver.push::<api::Lod>();
    resolver.push::<api::Object3d>();
    resolver.push::<api::Window>();
    resolver.push::<api::World>();
    resolver.push::<api::NodeMw>();

    // gamez nodes pm
    resolver.push::<api::LodPm>();
    resolver.push::<api::Object3dPm>();
    resolver.push::<api::NodePm>();

    // gamez mechlib
    resolver.push::<api::ModelMw>();
    resolver.push::<api::ModelPm>();

    // gamez mod
    resolver.push::<api::GameZMwMetadata>();
    resolver.push::<api::GameZMwData>();
    resolver.push::<api::GameZPmMetadata>();
    resolver.push::<api::GameZPmData>();
    resolver.push::<api::GameZCsMetadata>();
    resolver.push::<api::GameZCsData>();
    resolver.push::<api::GameZRcMetadata>();
    resolver.push::<api::GameZRcData>();

    // anim mod
    resolver.push::<api::AnimName>();
    resolver.push::<api::AnimPtr>();
    resolver.push::<api::AnimMetadata>();
    resolver.push::<api::AnimActivation>();
    resolver.push::<api::Execution>();
    resolver.push::<api::NamePad>();
    resolver.push::<api::NamePtr>();
    resolver.push::<api::NamePtrFlags>();
    resolver.push::<api::SeqActivation>();
    resolver.push::<api::PrereqAnimation>();
    resolver.push::<api::PrereqObject>();
    resolver.push::<api::PrereqParent>();
    resolver.push::<api::ActivationPrereq>();

    // anim events
    resolver.push::<api::AtNode>();
    resolver.push::<api::StopAnimation>();
    resolver.push::<api::ResetAnimation>();
    resolver.push::<api::InvalidateAnimation>();
    resolver.push::<api::CallAnimationAtNode>();
    resolver.push::<api::CallAnimationWithNode>();
    resolver.push::<api::CallAnimationTargetNode>();
    resolver.push::<api::CallAnimationParameters>();
    resolver.push::<api::CallAnimation>();
    resolver.push::<api::CallObjectConnector>();
    resolver.push::<api::Loop>();
    resolver.push::<api::RandomWeightCond>();
    resolver.push::<api::PlayerRangeCond>();
    resolver.push::<api::AnimationLodCond>();
    resolver.push::<api::HwRenderCond>();
    resolver.push::<api::PlayerFirstPersonCond>();
    resolver.push::<api::If>();
    resolver.push::<api::ElseIf>();
    resolver.push::<api::Else>();
    resolver.push::<api::EndIf>();
    resolver.push::<api::Callback>();
    resolver.push::<api::DetonateWeapon>();
    resolver.push::<api::Rgba>();
    resolver.push::<api::FrameBufferEffectColor>();
    resolver.push::<api::FogType>();
    resolver.push::<api::FogState>();
    resolver.push::<api::LightAnimation>();
    resolver.push::<api::LightState>();
    resolver.push::<api::ObjectActiveState>();
    resolver.push::<api::ObjectAddChild>();
    resolver.push::<api::ObjectConnector>();
    resolver.push::<api::ObjectCycleTexture>();
    resolver.push::<api::FloatFromTo>();
    resolver.push::<api::Vec3FromTo>();
    resolver.push::<api::ObjectMotionFromTo>();
    resolver.push::<api::TranslateData>();
    resolver.push::<api::RotateData>();
    resolver.push::<api::ScaleData>();
    resolver.push::<api::ObjectMotionSiFrame>();
    resolver.push::<api::ObjectMotionSiScript>();
    resolver.push::<api::GravityMode>();
    resolver.push::<api::Gravity>();
    resolver.push::<api::ForwardRotationTime>();
    resolver.push::<api::ForwardRotationDistance>();
    resolver.push::<api::ForwardRotation>();
    resolver.push::<api::XyzRotation>();
    resolver.push::<api::ObjectMotionTranslation>();
    resolver.push::<api::ObjectMotionScale>();
    resolver.push::<api::BounceSequence>();
    resolver.push::<api::BounceSound>();
    resolver.push::<api::ObjectMotion>();
    resolver.push::<api::ObjectOpacity>();
    resolver.push::<api::ObjectOpacityFromTo>();
    resolver.push::<api::ObjectOpacityState>();
    resolver.push::<api::RotateState>();
    resolver.push::<api::ObjectRotateState>();
    resolver.push::<api::ObjectScaleState>();
    resolver.push::<api::ObjectTranslateState>();
    resolver.push::<api::IntervalType>();
    resolver.push::<api::Interval>();
    resolver.push::<api::PufferStateCycleTextures>();
    resolver.push::<api::PufferState>();
    resolver.push::<api::CallSequence>();
    resolver.push::<api::StopSequence>();
    resolver.push::<api::SoundNode>();
    resolver.push::<api::Sound>();
    resolver.push::<api::EventData>();
    resolver.push::<api::StartOffset>();
    resolver.push::<api::EventStart>();
    resolver.push::<api::Event>();

    // anim mod
    resolver.push::<api::SeqDef>();
    resolver.push::<api::ResetState>();
    resolver.push::<api::AnimDef>();

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
