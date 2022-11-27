macro_rules! type_map {
    ($($name:ident => $num:literal,)+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum TypeMap {
            $($name,)+
        }

        impl TypeMap {
            #[inline]
            pub const fn from_u32(value: u32) -> Option<Self> {
                match value {
                    $($num => Some(Self::$name),)+
                    _ => None,
                }
            }

            #[inline]
            pub const fn to_u32(&self) -> u32 {
                match self {
                    $(Self::$name => $num,)+
                }
            }
        }

        impl Into<String> for TypeMap {
            fn into(self) -> String {
                match self {
                    $(Self::$name => String::from(stringify!($name)),)+
                }
            }
        }
    };
}

type_map! {
    U8 => 10,
    U16 => 11,
    U32 => 12,
    I8 => 20,
    I16 => 21,
    I32 => 22,
    F32 => 30,
    BoolTrue => 40,
    BoolFalse => 41,
    None => 32,
    Some => 33,
    Str => 50,
    Bytes => 51,
    Seq => 60,
    Struct => 70,
    // Map => 71,
    EnumUnit => 80,
    EnumNewType => 81,
}
