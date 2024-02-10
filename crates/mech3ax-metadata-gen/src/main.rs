mod csharp_type;
mod enums;
mod fields;
mod module_path;
mod resolver;
mod structs;
mod templates;
mod unions;

use mech3ax_api_types as api;
use resolver::TypeResolver;

fn main() {
    let mut resolver = TypeResolver::new();

    // --- types.rs
    resolver.push::<api::Range>();
    resolver.push::<api::Vec3>();
    resolver.push::<api::Color>();
    resolver.push::<api::Quaternion>();
    resolver.push::<api::Matrix>();

    // --- zmap.rs
    resolver.push::<api::zmap::MapColor>();
    resolver.push::<api::zmap::MapVertex>();
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

    // --- archive.rs
    resolver.push::<api::archive::ArchiveEntry>();

    // --- image.rs
    resolver.push::<api::image::TextureAlpha>();
    resolver.push::<api::image::TextureStretch>();
    resolver.push::<api::image::PaletteData>();
    resolver.push::<api::image::GlobalPalette>();
    resolver.push::<api::image::TexturePalette>();
    resolver.push::<api::image::TextureInfo>();
    resolver.push::<api::image::TextureManifest>();

    // --- GameZ

    // --- gamez/materials.rs
    resolver.push::<api::gamez::materials::ColoredMaterial>();
    resolver.push::<api::gamez::materials::CycleData>();
    resolver.push::<api::gamez::materials::TexturedMaterial>();
    resolver.push::<api::gamez::materials::Material>();

    // --- gamez/mesh/mod.rs
    resolver.push::<api::gamez::mesh::UvCoord>();
    resolver.push::<api::gamez::mesh::MeshLight>();

    // --- gamez/mesh/mw.rs
    resolver.push::<api::gamez::mesh::PolygonMw>();
    resolver.push::<api::gamez::mesh::MeshMw>();

    // --- gamez/mesh/ng.rs
    resolver.push::<api::gamez::mesh::PolygonFlags>();
    resolver.push::<api::gamez::mesh::MeshMaterialInfo>();
    resolver.push::<api::gamez::mesh::PolygonMaterialNg>();
    resolver.push::<api::gamez::mesh::PolygonNg>();
    resolver.push::<api::gamez::mesh::MeshNg>();

    // --- gamez/mesh/rc.rs
    resolver.push::<api::gamez::mesh::PolygonRc>();
    resolver.push::<api::gamez::mesh::MeshRc>();

    // --- Nodes (required for GameZ mechlib)

    // --- nodes/mod.rs
    resolver.push::<api::nodes::Camera>();
    resolver.push::<api::nodes::Display>();
    resolver.push::<api::nodes::Window>();
    resolver.push::<api::nodes::AreaPartition>();
    resolver.push::<api::nodes::Area>();
    resolver.push::<api::nodes::BoundingBox>();
    resolver.push::<api::nodes::Transformation>();
    resolver.push::<api::nodes::PartitionPg>();
    resolver.push::<api::nodes::PartitionValue>();
    resolver.push::<api::nodes::PartitionNg>();
    resolver.push::<api::nodes::NodeFlags>();

    // --- nodes/mw.rs
    resolver.push::<api::nodes::mw::Empty>();
    resolver.push::<api::nodes::mw::Light>();
    resolver.push::<api::nodes::mw::Lod>();
    resolver.push::<api::nodes::mw::Object3d>();
    resolver.push::<api::nodes::mw::World>();
    resolver.push::<api::nodes::mw::NodeMw>();

    // --- nodes/pm.rs
    resolver.push::<api::nodes::pm::AreaPartitionPm>();
    resolver.push::<api::nodes::pm::Light>();
    resolver.push::<api::nodes::pm::Lod>();
    resolver.push::<api::nodes::pm::Object3d>();
    resolver.push::<api::nodes::pm::World>();
    resolver.push::<api::nodes::pm::NodePm>();

    // --- nodes/cs.rs (requires AreaPartitionPm?)
    resolver.push::<api::nodes::cs::Camera>();
    resolver.push::<api::nodes::cs::Light>();
    resolver.push::<api::nodes::cs::Lod>();
    resolver.push::<api::nodes::cs::Object3d>();
    resolver.push::<api::nodes::cs::Window>();
    resolver.push::<api::nodes::cs::World>();
    resolver.push::<api::nodes::cs::NodeCs>();

    // --- nodes/rc.rs
    resolver.push::<api::nodes::rc::RotationTranslation>();
    resolver.push::<api::nodes::rc::TranslationOnly>();
    resolver.push::<api::nodes::rc::Transformation>();
    resolver.push::<api::nodes::rc::Empty>();
    resolver.push::<api::nodes::rc::Light>();
    resolver.push::<api::nodes::rc::Lod>();
    resolver.push::<api::nodes::rc::Object3d>();
    resolver.push::<api::nodes::rc::World>();
    resolver.push::<api::nodes::rc::NodeRc>();

    // --- gamez/mechlib.rs
    resolver.push::<api::gamez::mechlib::ModelMw>();
    resolver.push::<api::gamez::mechlib::ModelPm>();

    // --- gamez/mod.rs
    resolver.push::<api::gamez::GameZMetadataMw>();
    resolver.push::<api::gamez::GameZDataMw>();
    resolver.push::<api::gamez::GameZMetadataPm>();
    resolver.push::<api::gamez::GameZDataPm>();
    resolver.push::<api::gamez::GameZMetadataCs>();
    resolver.push::<api::gamez::TextureName>();
    resolver.push::<api::gamez::GameZDataCs>();
    resolver.push::<api::gamez::GameZDataRc>();

    // --- Anim

    // --- anim/events.rs
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

    // --- anim/mod.rs
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

    let env = templates::make_env();
    let (enums, structs, unions, mut directories) = resolver.into_values();
    directories.sort();

    for path in directories {
        std::fs::create_dir(&path)
            .unwrap_or_else(|e| panic!("failed to create `{}`: {:?}", path.display(), e));
    }

    for item in enums {
        let contents = item.render_impl(&env).unwrap();
        std::fs::write(&item.path, contents)
            .unwrap_or_else(|e| panic!("failed to write `{}`: {:?}", item.path.display(), e));
    }

    for item in structs {
        let contents = item.render_impl(&env).unwrap();
        std::fs::write(&item.path, contents)
            .unwrap_or_else(|e| panic!("failed to write `{}`: {:?}", item.path.display(), e));
    }

    for item in unions {
        let contents = item.render_impl(&env).unwrap();
        std::fs::write(&item.path, contents)
            .unwrap_or_else(|e| panic!("failed to write `{}`: {:?}", item.path.display(), e));
    }
}
