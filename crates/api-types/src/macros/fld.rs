macro_rules! fld {
    (
        $(#[doc = $struct_doc:literal])*
        struct $name:ident $(: $semantic:tt)? {$(
            $(#[serde($($serde:tt)*)])?
            $(#[doc = $doc:literal])*
            $field_name:ident: $field_ty:ty,
        )*}
    ) => {
        $(#[doc = $struct_doc])*
        #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        pub struct $name {$(
            $(#[serde($($serde)*)])?
            pub $field_name: $field_ty,
        )*}

        impl ::mech3ax_metadata_types::DerivedMetadata for $name {
            const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
                &::mech3ax_metadata_types::TypeInfo::Struct(::mech3ax_metadata_types::TypeInfoStruct {
                    name: stringify!($name),
                    fields: &[$(
                        ::mech3ax_metadata_types::TypeInfoStructField {
                            name: stringify!($field_name),
                            type_info: <$field_ty as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO,
                            default: $crate::fld!(@serde $($($serde)*)?),
                        },
                    )*],
                    module_path: ::std::module_path!(),
                    dotnet: ::mech3ax_metadata_types::TypeInfoStructDotNet {
                        // semantic: #semantic,
                        // generics: #generics,
                        // partial: #partial,
                        // namespace: #namespace,
                        semantic: $crate::fld!(@sem $($semantic)?),
                        generics: None,
                        partial: false,
                        namespace: None,
                    },
                });
        }
    };
    (@sem ) => {
        ::mech3ax_metadata_types::TypeSemantic::Ref
    };
    (@sem Val) => {
        ::mech3ax_metadata_types::TypeSemantic::Val
    };
    (@sem Ref) => {
        ::mech3ax_metadata_types::TypeSemantic::Ref
    };
    (@serde ) => {
        ::mech3ax_metadata_types::DefaultHandling::Normal
    };
    (@serde with = "bytes") => {
        ::mech3ax_metadata_types::DefaultHandling::Normal
    };
    (@serde skip_serializing_if = "Option::is_none", default) => {
        ::mech3ax_metadata_types::DefaultHandling::OptionIsNone
    };
    (@serde skip_serializing_if = "Soil::is_default", default) => {
        ::mech3ax_metadata_types::DefaultHandling::SoilIsDefault
    };
    (@serde skip_serializing_if = "i32_is_neg_one", default = "i32_neg_one") => {
        ::mech3ax_metadata_types::DefaultHandling::I32IsNegOne
    };
    (@serde skip_serializing_if = "pointer_zero", default) => {
        ::mech3ax_metadata_types::DefaultHandling::PointerZero
    };
}
pub(crate) use fld;
