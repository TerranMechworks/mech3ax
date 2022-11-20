mod csharp_type;
mod enums;
mod fields;
mod module_path;
mod options;
mod resolver;
mod structs;
mod templates;
mod unions;

use mech3ax_api_types as api;
use resolver::TypeResolver;

fn main() {
    let mut resolver = TypeResolver::new();

    resolver.push::<api::Range>();
    resolver.push::<api::Vec3>();
    resolver.push::<api::Color>();
    resolver.push::<api::Quaternion>();
    resolver.push::<api::Matrix>();

    resolver.push::<api::archive::ArchiveEntry>();

    resolver.push::<api::image::TextureAlpha>();
    resolver.push::<api::image::TextureStretch>();
    resolver.push::<api::image::PaletteData>();
    resolver.push::<api::image::GlobalPalette>();
    resolver.push::<api::image::TexturePalette>();
    resolver.push::<api::image::TextureInfo>();
    resolver.push::<api::image::TextureManifest>();

    resolver.push::<api::interp::Script>();

    resolver.push::<api::messages::MessageEntry>();
    resolver.push::<api::messages::Messages>();

    resolver.push::<api::motion::MotionFrame>();
    resolver.push::<api::motion::MotionPart>();
    resolver.push::<api::motion::Motion>();

    resolver.push::<api::zmap::MapColor>();
    resolver.push::<api::zmap::MapVertex>();
    resolver.push::<api::zmap::MapFeature>();
    resolver.push::<api::zmap::MapRc>();

    resolver.push::<api::gamez::ColoredMaterial>();
    resolver.push::<api::gamez::CycleData>();
    resolver.push::<api::gamez::TexturedMaterial>();
    resolver.push::<api::gamez::Material>();

    // gamez mesh
    resolver.push::<api::gamez::UvCoord>();
    resolver.push::<api::gamez::MeshLight>();
    resolver.push::<api::gamez::PolygonMw>();
    resolver.push::<api::gamez::MeshMw>();
    resolver.push::<api::gamez::PolygonFlags>();
    resolver.push::<api::gamez::MeshTexture>();
    resolver.push::<api::gamez::PolygonTextureNg>();
    resolver.push::<api::gamez::PolygonNg>();
    resolver.push::<api::gamez::MeshNg>();
    resolver.push::<api::gamez::PolygonRc>();
    resolver.push::<api::gamez::MeshRc>();

    // gamez nodes
    resolver.push::<api::gamez::AreaPartition>();
    resolver.push::<api::gamez::Area>();
    resolver.push::<api::gamez::BoundingBox>();
    resolver.push::<api::gamez::Transformation>();
    resolver.push::<api::gamez::Partition>();
    resolver.push::<api::gamez::NodeFlags>();

    // gamez nodes mw
    resolver.push::<api::gamez::Camera>();
    resolver.push::<api::gamez::Display>();
    resolver.push::<api::gamez::Empty>();
    resolver.push::<api::gamez::Light>();
    resolver.push::<api::gamez::Lod>();
    resolver.push::<api::gamez::Object3d>();
    resolver.push::<api::gamez::Window>();
    resolver.push::<api::gamez::World>();
    resolver.push::<api::gamez::NodeMw>();

    // gamez nodes pm
    resolver.push::<api::gamez::LodPm>();
    resolver.push::<api::gamez::Object3dPm>();
    resolver.push::<api::gamez::NodePm>();

    // gamez mechlib
    resolver.push::<api::gamez::ModelMw>();
    resolver.push::<api::gamez::ModelPm>();

    // gamez mod
    resolver.push::<api::gamez::GameZMwMetadata>();
    resolver.push::<api::gamez::GameZMwData>();
    resolver.push::<api::gamez::GameZPmMetadata>();
    resolver.push::<api::gamez::GameZPmData>();
    resolver.push::<api::gamez::GameZCsMetadata>();
    resolver.push::<api::gamez::GameZCsData>();
    resolver.push::<api::gamez::GameZRcMetadata>();
    resolver.push::<api::gamez::GameZRcData>();

    // anim mod
    resolver.push::<api::anim::AnimName>();
    resolver.push::<api::anim::AnimPtr>();
    resolver.push::<api::anim::AnimMetadata>();
    resolver.push::<api::anim::AnimActivation>();
    resolver.push::<api::anim::Execution>();
    resolver.push::<api::anim::NamePad>();
    resolver.push::<api::anim::NamePtr>();
    resolver.push::<api::anim::NamePtrFlags>();
    resolver.push::<api::anim::SeqActivation>();
    resolver.push::<api::anim::PrereqAnimation>();
    resolver.push::<api::anim::PrereqObject>();
    resolver.push::<api::anim::PrereqParent>();
    resolver.push::<api::anim::ActivationPrereq>();

    // anim events
    resolver.push::<api::anim::AtNode>();
    resolver.push::<api::anim::StopAnimation>();
    resolver.push::<api::anim::ResetAnimation>();
    resolver.push::<api::anim::InvalidateAnimation>();
    resolver.push::<api::anim::CallAnimationAtNode>();
    resolver.push::<api::anim::CallAnimationWithNode>();
    resolver.push::<api::anim::CallAnimationTargetNode>();
    resolver.push::<api::anim::CallAnimationParameters>();
    resolver.push::<api::anim::CallAnimation>();
    resolver.push::<api::anim::CallObjectConnector>();
    resolver.push::<api::anim::Loop>();
    resolver.push::<api::anim::RandomWeightCond>();
    resolver.push::<api::anim::PlayerRangeCond>();
    resolver.push::<api::anim::AnimationLodCond>();
    resolver.push::<api::anim::HwRenderCond>();
    resolver.push::<api::anim::PlayerFirstPersonCond>();
    resolver.push::<api::anim::If>();
    resolver.push::<api::anim::ElseIf>();
    resolver.push::<api::anim::Else>();
    resolver.push::<api::anim::EndIf>();
    resolver.push::<api::anim::Callback>();
    resolver.push::<api::anim::DetonateWeapon>();
    resolver.push::<api::anim::Rgba>();
    resolver.push::<api::anim::FrameBufferEffectColor>();
    resolver.push::<api::anim::FogType>();
    resolver.push::<api::anim::FogState>();
    resolver.push::<api::anim::LightAnimation>();
    resolver.push::<api::anim::LightState>();
    resolver.push::<api::anim::ObjectActiveState>();
    resolver.push::<api::anim::ObjectAddChild>();
    resolver.push::<api::anim::ObjectConnector>();
    resolver.push::<api::anim::ObjectCycleTexture>();
    resolver.push::<api::anim::FloatFromTo>();
    resolver.push::<api::anim::Vec3FromTo>();
    resolver.push::<api::anim::ObjectMotionFromTo>();
    resolver.push::<api::anim::TranslateData>();
    resolver.push::<api::anim::RotateData>();
    resolver.push::<api::anim::ScaleData>();
    resolver.push::<api::anim::ObjectMotionSiFrame>();
    resolver.push::<api::anim::ObjectMotionSiScript>();
    resolver.push::<api::anim::GravityMode>();
    resolver.push::<api::anim::Gravity>();
    resolver.push::<api::anim::ForwardRotationTime>();
    resolver.push::<api::anim::ForwardRotationDistance>();
    resolver.push::<api::anim::ForwardRotation>();
    resolver.push::<api::anim::XyzRotation>();
    resolver.push::<api::anim::ObjectMotionTranslation>();
    resolver.push::<api::anim::ObjectMotionScale>();
    resolver.push::<api::anim::BounceSequence>();
    resolver.push::<api::anim::BounceSound>();
    resolver.push::<api::anim::ObjectMotion>();
    resolver.push::<api::anim::ObjectOpacity>();
    resolver.push::<api::anim::ObjectOpacityFromTo>();
    resolver.push::<api::anim::ObjectOpacityState>();
    resolver.push::<api::anim::RotateState>();
    resolver.push::<api::anim::ObjectRotateState>();
    resolver.push::<api::anim::ObjectScaleState>();
    resolver.push::<api::anim::ObjectTranslateState>();
    resolver.push::<api::anim::IntervalType>();
    resolver.push::<api::anim::Interval>();
    resolver.push::<api::anim::PufferStateCycleTextures>();
    resolver.push::<api::anim::PufferState>();
    resolver.push::<api::anim::CallSequence>();
    resolver.push::<api::anim::StopSequence>();
    resolver.push::<api::anim::SoundNode>();
    resolver.push::<api::anim::Sound>();
    resolver.push::<api::anim::EventData>();
    resolver.push::<api::anim::StartOffset>();
    resolver.push::<api::anim::EventStart>();
    resolver.push::<api::anim::Event>();

    // anim mod
    resolver.push::<api::anim::SeqDef>();
    resolver.push::<api::anim::ResetState>();
    resolver.push::<api::anim::AnimDef>();

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
