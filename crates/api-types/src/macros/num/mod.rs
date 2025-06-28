macro_rules! num {
    (
        $(#[doc = $enum_doc:literal])*
        enum $name:ident : $ty:tt {$(
            $(#[doc = $variant_doc:literal])*
            $variant:ident = $val:literal,
        )+}
    ) => {
        ::mech3ax_types::primitive_enum! {
            $(#[doc = $enum_doc])*
            pub enum $name : $ty {$(
                $(#[doc = $variant_doc])*
                $variant = $val,
            )+}
        }

        #[automatically_derived]
        impl ::serde::ser::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                match *self {$(
                    $name::$variant => ::serde::ser::Serializer::serialize_unit_variant(
                        serializer,
                        stringify!($name),
                        $val,
                        stringify!($variant),
                    ),
                )+}
            }
        }

        #[automatically_derived]
        impl<'de> ::serde::de::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[repr(transparent)]
                struct Field($name);
                struct FieldVisitor;

                impl<'de> ::serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    #[inline]
                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        ::std::fmt::Formatter::write_str(formatter, "variant identifier")
                    }

                    fn visit_u32<E>(self, value: u32) -> ::std::result::Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            $($val => Ok(Field($name::$variant)),)+
                            _ => Err(::serde::de::Error::invalid_value(
                                ::serde::de::Unexpected::Unsigned(value.into()),
                                &"invalid variant index",
                            )),
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> ::std::result::Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            $(stringify!($variant) => Ok(Field($name::$variant)),)+
                            _ => Err(::serde::de::Error::unknown_variant(value, VARIANTS)),
                        }
                    }
                }

                impl<'de> ::serde::de::Deserialize<'de> for Field {
                    #[inline]
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                    where
                        D: ::serde::de::Deserializer<'de>,
                    {
                        ::serde::de::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
                    }
                }

                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    #[inline]
                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        ::std::fmt::Formatter::write_str(formatter, concat!("enum ", stringify!($name)))
                    }

                    fn visit_enum<A>(
                        self,
                        data: A,
                    ) -> ::std::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::EnumAccess<'de>,
                    {
                        let (field, variant): (Field, _) = ::serde::de::EnumAccess::variant(data)?;
                        ::serde::de::VariantAccess::unit_variant(variant)?;
                        Ok(field.0)
                    }
                }

                const VARIANTS: &'static [&'static str] = &[
                    $(stringify!($variant),)+
                ];
                ::serde::de::Deserializer::deserialize_enum(
                    deserializer,
                    stringify!($name),
                    VARIANTS,
                    Visitor,
                )
            }
        }

        #[automatically_derived]
        impl ::mech3ax_metadata_types::DerivedMetadata for $name {
            const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
                &::mech3ax_metadata_types::TypeInfo::Enum(::mech3ax_metadata_types::TypeInfoEnum {
                    name: stringify!($name),
                    variants: &[
                        $((stringify!($variant), $val),)+
                    ],
                    module_path: ::std::module_path!(),
                });
        }
    };
    (
        $(#[doc = $enum_doc:literal])*
        enum $name:ident {$(
            $(#[doc = $variant_doc:literal])*
            $variant:ident = $val:literal,
        )+}
    ) => {
        $(#[doc = $enum_doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {$(
            $(#[doc = $variant_doc])*
            $variant = $val,
        )+}

        #[automatically_derived]
        impl ::serde::ser::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                match *self {$(
                    $name::$variant => ::serde::ser::Serializer::serialize_unit_variant(
                        serializer,
                        stringify!($name),
                        $val,
                        stringify!($variant),
                    ),
                )+}
            }
        }

        #[automatically_derived]
        impl<'de> ::serde::de::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[repr(transparent)]
                struct Field($name);
                struct FieldVisitor;

                impl<'de> ::serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    #[inline]
                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        ::std::fmt::Formatter::write_str(formatter, "variant identifier")
                    }

                    fn visit_u32<E>(self, value: u32) -> ::std::result::Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            $($val => Ok(Field($name::$variant)),)+
                            _ => Err(::serde::de::Error::invalid_value(
                                ::serde::de::Unexpected::Unsigned(value.into()),
                                &"invalid variant index",
                            )),
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> ::std::result::Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            $(stringify!($variant) => Ok(Field($name::$variant)),)+
                            _ => Err(::serde::de::Error::unknown_variant(value, VARIANTS)),
                        }
                    }
                }

                impl<'de> ::serde::de::Deserialize<'de> for Field {
                    #[inline]
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                    where
                        D: ::serde::de::Deserializer<'de>,
                    {
                        ::serde::de::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
                    }
                }

                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    #[inline]
                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        ::std::fmt::Formatter::write_str(formatter, concat!("enum ", stringify!($name)))
                    }

                    fn visit_enum<A>(
                        self,
                        data: A,
                    ) -> ::std::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::EnumAccess<'de>,
                    {
                        let (field, variant): (Field, _) = ::serde::de::EnumAccess::variant(data)?;
                        ::serde::de::VariantAccess::unit_variant(variant)?;
                        Ok(field.0)
                    }
                }

                const VARIANTS: &'static [&'static str] = &[
                    $(stringify!($variant),)+
                ];
                ::serde::de::Deserializer::deserialize_enum(
                    deserializer,
                    stringify!($name),
                    VARIANTS,
                    Visitor,
                )
            }
        }

        #[automatically_derived]
        impl ::mech3ax_metadata_types::DerivedMetadata for $name {
            const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
                &::mech3ax_metadata_types::TypeInfo::Enum(::mech3ax_metadata_types::TypeInfoEnum {
                    name: stringify!($name),
                    variants: &[
                        $((stringify!($variant), $val),)+
                    ],
                    module_path: ::std::module_path!(),
                });
        }
    };
}
pub(crate) use num;

#[cfg(test)]
mod tests;
