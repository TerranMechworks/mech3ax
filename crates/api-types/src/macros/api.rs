macro_rules! api {
    (
        $(#[doc = $struct_doc:literal])*
        struct $name:ident $(: $semantic:tt)? {$(
            $(#[doc = $field_doc:literal])*
            $(#[serde($($serde:tt)*)])?
            $field_name:ident: $field_ty:ty $(= { $($default:tt)* })?,
        )*}
    ) => {
        $(#[doc = $struct_doc])*
        #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        pub struct $name {$(
            $(#[doc = $field_doc])*
            $(#[serde($($serde)*)])?
            pub $field_name: $field_ty,
        )*}

        $crate::api!(@md $name { $($field_name: $field_ty,)* });
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
    (@default ) => {
        ::mech3ax_metadata_types::DefaultHandling::Normal
    };
    (@default None) => {
        ::mech3ax_metadata_types::DefaultHandling::OptionIsNone
    };
    (@default false) => {
        ::mech3ax_metadata_types::DefaultHandling::BoolIsFalse
    };
    (@default true) => {
        ::mech3ax_metadata_types::DefaultHandling::BoolIsTrue
    };
    (@default Soil::Default) => {
        ::mech3ax_metadata_types::DefaultHandling::SoilIsDefault
    };
    (@default -1i32) => {
        ::mech3ax_metadata_types::DefaultHandling::I32IsNegOne
    };
    (@default 0u32) => {
        ::mech3ax_metadata_types::DefaultHandling::PointerIsZero
    };
    (@serde None) => {
        #[serde(skip_serializing_if = "Option::is_none", default)]
    };
    (@serde Soil::Default) => {
        #[serde(skip_serializing_if = "Soil::is_default", default)]
    };
    (@serde -1i32) => {
        #[serde(skip_serializing_if = "::crate::serde::i32_is_neg_one", default = "::crate::serde::i32_neg_one")]
    };
    (@serde 0u32) => {
        #[serde(skip_serializing_if = "::crate::serde::pointer_zero", default)]
    };
    (
        $(#[doc = $struct_doc:literal])*
        #[repr(C)]
        struct $name:ident $(: $semantic:tt)? {$(
            $(#[doc = $field_doc:literal])*
            $field_name:ident: $field_ty:ty,
        )*}
    ) => {
        $(#[doc = $struct_doc])*
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            ::bytemuck::AnyBitPattern,
            ::bytemuck::NoUninit,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::mech3ax_types::Offsets,
        )]
        #[repr(C)]
        pub struct $name {$(
            $(#[doc = $field_doc])*
            pub $field_name: $field_ty,
        )*}

        $crate::api!(@md $name $(: $semantic)? { $($field_name: $field_ty,)* });
    };
    (@md $name:ident $(: $semantic:tt)? {$(
        $field_name:ident: $field_ty:ty,
    )*}) => {
        impl ::mech3ax_metadata_types::DerivedMetadata for $name {
            const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
                &::mech3ax_metadata_types::TypeInfo::Struct(::mech3ax_metadata_types::TypeInfoStruct {
                    name: stringify!($name),
                    semantic: $crate::api!(@sem $($semantic)?),
                    fields: &[$(
                        ::mech3ax_metadata_types::TypeInfoStructField {
                            name: stringify!($field_name),
                            type_info: <$field_ty as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO,
                            default: ::mech3ax_metadata_types::DefaultHandling::Normal,
                        },
                    )*],
                    module_path: ::std::module_path!(),
                });
        }
    }
}
pub(crate) use api;
