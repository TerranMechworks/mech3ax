use crate::maybe::{PrimitiveRepr, SupportsMaybe};
use std::fmt;

/// A trait for enums that can be converted from/to a primitive type (e.g. u32).
/// This trait should not be implemented manually; instead it is intended to be
/// derived by the corresponding macro!
///
/// Three methods are derived:
/// * `fn PrimitiveEnum::from_primitive(v: <primitive>) -> Option<Self>` for
///   trying to convert a primitive type to the enum.
/// * `From<Self> for <primitive>` for converting the enum to a primitive type
///   (`into()`).
pub trait PrimitiveEnum<R>
where
    R: PrimitiveRepr,
    Self: Clone
        + Copy
        + PartialEq
        + Eq
        + fmt::Debug
        + Sized
        + Sync
        + Send
        + 'static
        + SupportsMaybe<R>,
{
    const DISCRIMINANTS: &'static [u32];
}

#[macro_export]
macro_rules! primitive_enum {
    ($(#[$outer:meta])* $vis:vis enum $name:ident : $ty:tt {
        $(
            $(#[$inner:meta])*
            $variant:ident = $val:literal,
        )+
    }) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr($ty)]
        $vis enum $name {
            $(
                $(#[$inner])*
                $variant = $val,
            )+
        }

        impl $name {
            #[inline]
            pub const fn from_bits(v: $ty) -> ::core::option::Option<Self> {
                match v {
                    $($val => ::core::option::Option::Some(Self::$variant),)+
                    _ => ::core::option::Option::None,
                }
            }

            #[inline]
            pub const fn maybe(self) -> $crate::maybe::Maybe<$ty, Self> {
                $crate::maybe::Maybe::new(self as _)
            }
        }

        impl $crate::maybe::SupportsMaybe<$ty> for $name {
            #[inline]
            fn from_bits(v: $ty) -> ::core::option::Option<Self> {
                Self::from_bits(v)
            }

            #[inline]
            fn fmt_value(v: $ty, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match Self::from_bits(v) {
                    ::core::option::Option::Some(e) => ::core::write!(f, "{:?} ({})", e, v),
                    ::core::option::Option::None => ::core::write!(f, "<unknown> ({})", v),
                }
            }

            #[inline]
            fn maybe(self) -> $crate::maybe::Maybe<$ty, $name> {
                Self::maybe(self)
            }
        }

        impl $crate::primitive_enum::PrimitiveEnum<$ty> for $name {
            const DISCRIMINANTS: &'static [u32] = &[
                $($val,)+
            ];
        }
    };
}

fn is_consecutive(first: u32, middle: &[u32], last: u32) -> bool {
    let mut curr = first + 1;
    for v in middle.iter().copied() {
        if curr != v {
            return false;
        }
        curr += 1;
    }
    curr == last
}

fn format_discriminants_more(first: u32, middle: &[u32], last: u32) -> String {
    if is_consecutive(first, middle, last) {
        format!("{}..{}", first, last)
    } else {
        use std::fmt::Write as _;
        let mut s = String::new();
        write!(s, "{}, ", first).unwrap();
        for v in middle {
            write!(s, "{}, ", v).unwrap();
        }
        write!(s, "or {}", last).unwrap();
        s
    }
}

pub fn format_discriminants(discriminants: &[impl Into<u32> + Copy]) -> String {
    let nums: Vec<u32> = discriminants.iter().copied().map(Into::into).collect();
    match &nums[..] {
        [] => "<no valid options>".to_string(),
        [one] => format!("{}", one),
        [one, two] => format!("{} or {}", one, two),
        [first, middle @ .., last] => format_discriminants_more(*first, middle, *last),
    }
}

#[cfg(test)]
mod tests;
