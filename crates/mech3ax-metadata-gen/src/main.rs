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

    resolver.push::<api::gamez::materials::ColoredMaterial>();
    resolver.push::<api::gamez::materials::CycleData>();
    resolver.push::<api::gamez::materials::TexturedMaterial>();
    resolver.push::<api::gamez::materials::Material>();

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

    resolver.push::<api::gamez::mechlib::ModelMw>();
    resolver.push::<api::gamez::mechlib::ModelPm>();

    // gamez mod
    resolver.push::<api::gamez::GameZMwMetadata>();
    resolver.push::<api::gamez::GameZMwData>();
    resolver.push::<api::gamez::GameZPmMetadata>();
    resolver.push::<api::gamez::GameZPmData>();
    resolver.push::<api::gamez::GameZCsMetadata>();
    resolver.push::<api::gamez::GameZCsData>();
    resolver.push::<api::gamez::GameZRcMetadata>();
    resolver.push::<api::gamez::GameZRcData>();

    resolver.push::<api::anim::events::AtNode>();
    resolver.push::<api::anim::events::StopAnimation>();
    resolver.push::<api::anim::events::ResetAnimation>();
    resolver.push::<api::anim::events::InvalidateAnimation>();
    resolver.push::<api::anim::events::CallAnimationAtNode>();
    resolver.push::<api::anim::events::CallAnimationWithNode>();
    resolver.push::<api::anim::events::CallAnimationTargetNode>();
    resolver.push::<api::anim::events::CallAnimationParameters>();
    resolver.push::<api::anim::events::CallAnimation>();
    resolver.push::<api::anim::events::CallObjectConnector>();
    resolver.push::<api::anim::events::Loop>();
    resolver.push::<api::anim::events::RandomWeightCond>();
    resolver.push::<api::anim::events::PlayerRangeCond>();
    resolver.push::<api::anim::events::AnimationLodCond>();
    resolver.push::<api::anim::events::HwRenderCond>();
    resolver.push::<api::anim::events::PlayerFirstPersonCond>();
    resolver.push::<api::anim::events::If>();
    resolver.push::<api::anim::events::ElseIf>();
    resolver.push::<api::anim::events::Else>();
    resolver.push::<api::anim::events::EndIf>();
    resolver.push::<api::anim::events::Callback>();
    resolver.push::<api::anim::events::DetonateWeapon>();
    resolver.push::<api::anim::events::Rgba>();
    resolver.push::<api::anim::events::FrameBufferEffectColor>();
    resolver.push::<api::anim::events::FogType>();
    resolver.push::<api::anim::events::FogState>();
    resolver.push::<api::anim::events::LightAnimation>();
    resolver.push::<api::anim::events::LightState>();
    resolver.push::<api::anim::events::ObjectActiveState>();
    resolver.push::<api::anim::events::ObjectAddChild>();
    resolver.push::<api::anim::events::ObjectConnector>();
    resolver.push::<api::anim::events::ObjectCycleTexture>();
    resolver.push::<api::anim::events::FloatFromTo>();
    resolver.push::<api::anim::events::Vec3FromTo>();
    resolver.push::<api::anim::events::ObjectMotionFromTo>();
    resolver.push::<api::anim::events::TranslateData>();
    resolver.push::<api::anim::events::RotateData>();
    resolver.push::<api::anim::events::ScaleData>();
    resolver.push::<api::anim::events::ObjectMotionSiFrame>();
    resolver.push::<api::anim::events::ObjectMotionSiScript>();
    resolver.push::<api::anim::events::GravityMode>();
    resolver.push::<api::anim::events::Gravity>();
    resolver.push::<api::anim::events::ForwardRotationTime>();
    resolver.push::<api::anim::events::ForwardRotationDistance>();
    resolver.push::<api::anim::events::ForwardRotation>();
    resolver.push::<api::anim::events::XyzRotation>();
    resolver.push::<api::anim::events::ObjectMotionTranslation>();
    resolver.push::<api::anim::events::ObjectMotionScale>();
    resolver.push::<api::anim::events::BounceSequence>();
    resolver.push::<api::anim::events::BounceSound>();
    resolver.push::<api::anim::events::ObjectMotion>();
    resolver.push::<api::anim::events::ObjectOpacity>();
    resolver.push::<api::anim::events::ObjectOpacityFromTo>();
    resolver.push::<api::anim::events::ObjectOpacityState>();
    resolver.push::<api::anim::events::RotateState>();
    resolver.push::<api::anim::events::ObjectRotateState>();
    resolver.push::<api::anim::events::ObjectScaleState>();
    resolver.push::<api::anim::events::ObjectTranslateState>();
    resolver.push::<api::anim::events::IntervalType>();
    resolver.push::<api::anim::events::Interval>();
    resolver.push::<api::anim::events::PufferStateCycleTextures>();
    resolver.push::<api::anim::events::PufferState>();
    resolver.push::<api::anim::events::CallSequence>();
    resolver.push::<api::anim::events::StopSequence>();
    resolver.push::<api::anim::events::SoundNode>();
    resolver.push::<api::anim::events::Sound>();
    resolver.push::<api::anim::events::EventData>();
    resolver.push::<api::anim::events::StartOffset>();
    resolver.push::<api::anim::events::EventStart>();
    resolver.push::<api::anim::events::Event>();

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
    resolver.push::<api::anim::ResetState>();
    resolver.push::<api::anim::SeqDef>();
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
