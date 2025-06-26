use super::resolver::TypeResolver;
use mech3ax_metadata_types::{DefaultHandling, TypeInfoStructField};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FieldSerde {
    pub(crate) serialize: String,
    pub(crate) deserialize: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Field {
    /// The struct field's JSON key name.
    pub(crate) key: &'static str,
    /// The struct field's Python field name.
    pub(crate) name: &'static str,
    /// The struct field's import definition, if it requires importing.
    pub(crate) import: Option<String>,
    /// The struct field's Python type.
    pub(crate) ty: Cow<'static, str>,
    /// The struct field's serialization information.
    pub(crate) serde: FieldSerde,
    /// The struct field's default value, if any.
    pub(crate) default: Option<String>,
}

fn rust_to_python_field_name(name: &'static str) -> &'static str {
    match name {
        // TODO: add more reserved keywords
        other => other,
    }
}

impl Field {
    pub(crate) fn new(
        struct_name: &'static str,
        field_info: &TypeInfoStructField,
        resolver: &TypeResolver,
    ) -> Self {
        // the JSON field name should match the type name, barring any serde
        // rename shenanigans.
        let key = field_info.name;
        let python_name = rust_to_python_field_name(field_info.name);

        let ty = resolver.resolve(field_info.type_info, struct_name);

        let serde = FieldSerde {
            serialize: ty.serde.make_serialize(),
            deserialize: ty.serde.make_deserialize(),
        };

        let default = match field_info.default {
            DefaultHandling::Normal => None,
            DefaultHandling::OptionIsNone => Some("None".to_string()),
            DefaultHandling::BoolFalse => Some("False".to_string()),
            DefaultHandling::BoolTrue => Some("True".to_string()),
            DefaultHandling::PointerZero => Some("0".to_string()),
            DefaultHandling::SoilIsDefault => {
                Some("mech3py.types.gamez.materials.Soil.Default".to_string())
            }
            DefaultHandling::I32IsNegOne => Some("-1".to_string()),
        };

        Self {
            key,
            name: python_name,
            import: ty.import.clone(),
            ty: ty.name.clone(),
            serde,
            default,
        }
    }
}
