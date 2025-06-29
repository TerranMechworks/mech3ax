macro_rules! sum {
    (
        $(#[doc = $sum_doc:literal])*
        enum $name:ident {$(
            $(#[doc = $variant_doc:literal])*
            $variant:ident $(($ty:ty))?,
        )+}
    ) => {
        $(#[doc = $sum_doc])*
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        pub enum $name {$(
            $(#[doc = $variant_doc])*
            $variant $(($ty))?,
        )+}

        impl ::mech3ax_metadata_types::DerivedMetadata for $name {
            const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
                &::mech3ax_metadata_types::TypeInfo::Union(::mech3ax_metadata_types::TypeInfoUnion {
                    name: stringify!($name),
                    variants: &[
                        $($crate::sum!(@variant $variant $($ty)?),)+
                    ],
                    module_path: ::std::module_path!(),
                });
        }
    };
    (@variant $name:ident $ty:ty) => {
        (stringify!($name), Some(<$ty as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO))
    };
    (@variant $name:ident) => {
        (stringify!($name), None)
    };
}
pub(crate) use sum;
