macro_rules! bit {
    (
        $(#[doc = $flags_doc:literal])*
        struct $name:ident : $ty:tt {$(
            $(#[serde($($serde:tt)*)])?
            $(#[doc = $variant_doc:literal])*
            const $flag:ident = 1 << $val:literal;
        )+}
    ) => {
        ::mech3ax_types::bitflags! {
            $(#[doc = $flags_doc])*
            pub struct $name : $ty {
            $(
                $(#[doc = $variant_doc])*
                const $flag = 1 << $val;
            )+
            }
        }

        ::mech3ax_types::json_flags! {
            struct $name {
            $(
                $(#[serde($($serde)*)])?
                $flag,
            )+
            }
        }

        #[automatically_derived]
        impl ::mech3ax_metadata_types::DerivedMetadata for $name {
            const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
                &::mech3ax_metadata_types::TypeInfo::Flags(::mech3ax_metadata_types::TypeInfoFlags {
                    name: stringify!($name),
                    repr: $crate::bit!(@repr $ty),
                    variants: &[
                        $((stringify!($flag), $val),)+
                    ],
                    module_path: ::std::module_path!(),
                });
        }

        #[automatically_derived]
        impl ::serde::ser::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {

                let v = *self;
                if serializer.is_human_readable() {
                    let json = v.exhaustive();
                    ::serde::ser::Serialize::serialize(&json, serializer)
                } else {
                    $crate::bit!(@ser $ty)(serializer, v.0)
                }
            }
        }

        #[automatically_derived]
        impl<'de> ::serde::de::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let json = ::serde::de::Deserialize::deserialize(deserializer)?;
                    Ok(Self::from_exhaustive(&json))
                } else {
                    let v = ::serde::de::Deserialize::deserialize(deserializer)?;
                    Self::from_bits(v).ok_or_else(|| {
                        let msg = format!(
                            concat!("Invalid value {} for ", stringify!($name)),
                            v
                        );
                        <<D as ::serde::de::Deserializer>::Error as ::serde::de::Error>::custom(msg)
                    })
                }
            }
        }
    };
    (@repr u8) => { ::mech3ax_metadata_types::TypeInfoFlagsRepr::U8 };
    (@repr u16) => { ::mech3ax_metadata_types::TypeInfoFlagsRepr::U16 };
    (@repr u32) => { ::mech3ax_metadata_types::TypeInfoFlagsRepr::U32 };
    (@ser u8) => { ::serde::ser::Serializer::serialize_u8 };
    (@ser u16) => { ::serde::ser::Serializer::serialize_u16 };
    (@ser u32) => { ::serde::ser::Serializer::serialize_u32 };
}
pub(crate) use bit;

#[cfg(test)]
mod tests;
