/// A trait for enums that can be converted from/to a primitive type (e.g. u32).
/// This trait should not be implemented manually; instead it is intended to be
/// derived by the corresponding macro!
///
/// Three methods are derived:
/// * `fn PrimitiveEnum::from_primitive(v: <primitive>) -> Option<Self>` for
///   trying to convert a primitive type to the enum.
/// * `From<Self> for <primitive>` for converting the enum to a primitive type
///   (`into()`).
/// * `const fn as_(self) -> <primitive>` for converting the enum to a
///   primitive type (`const`).
pub trait PrimitiveEnum: Sized + Sync + Send + 'static + Into<Self::Primitive> {
    type Primitive: Copy + std::fmt::Display + Into<u32>;
    const DISCRIMINANTS: &'static [Self::Primitive];

    fn from_primitive(v: Self::Primitive) -> Option<Self>;
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
            pub const fn as_(self) -> $ty {
                self as _
            }
        }

        impl From<$name> for $ty {
            #[inline]
            fn from(value: $name) -> Self {
                value as _
            }
        }

        impl $crate::primitive_enum::PrimitiveEnum for $name {
            type Primitive = $ty;
            const DISCRIMINANTS: &'static [Self::Primitive] = &[
                $($val,)+
            ];

            fn from_primitive(v: Self::Primitive) -> Option<Self> {
                match v {
                    $($val => Some(Self::$variant),)+
                    _ => None,
                }
            }
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
