use crate::enums::{ENUM_CONV, ENUM_IMPL};
use crate::options::{CONV_FACTORY, OPTIONS_IMPL};
use crate::structs::{STRUCT_GENERIC_CONV, STRUCT_IMPL, STRUCT_NORMAL_CONV};
use crate::unions::{UNION_CONV, UNION_IMPL};
use std::error::Error;
use tera::Tera;

macro_rules! tera_unwrap {
    ($stmt:expr) => {{
        if let Err(e) = $stmt {
            // with tera, the error source is very useful for debugging
            if let Some(source) = e.source() {
                eprintln!("{}", source);
            }
            panic!("{}", e);
        }
    }};
}

pub fn make_tera() -> Tera {
    let mut tera = Tera::default();
    tera_unwrap!(tera.add_raw_template("enum_impl.cs", ENUM_IMPL));
    tera_unwrap!(tera.add_raw_template("enum_conv.cs", ENUM_CONV));
    tera_unwrap!(tera.add_raw_template("struct_impl.cs", STRUCT_IMPL));
    tera_unwrap!(tera.add_raw_template("struct_normal_conv.cs", STRUCT_NORMAL_CONV));
    tera_unwrap!(tera.add_raw_template("struct_generic_conv.cs", STRUCT_GENERIC_CONV));
    tera_unwrap!(tera.add_raw_template("union_impl.cs", UNION_IMPL));
    tera_unwrap!(tera.add_raw_template("union_conv.cs", UNION_CONV));
    tera_unwrap!(tera.add_raw_template("options_impl.cs", OPTIONS_IMPL));
    tera_unwrap!(tera.add_raw_template("converter_factory.cs", CONV_FACTORY));
    tera
}
