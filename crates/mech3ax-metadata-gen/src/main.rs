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
