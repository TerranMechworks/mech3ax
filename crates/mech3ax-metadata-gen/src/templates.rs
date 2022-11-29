use crate::enums::ENUM_IMPL;
use crate::structs::{CLASS_IMPL, STRUCT_IMPL};
use crate::unions::UNION_IMPL;
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
    tera_unwrap!(tera.add_raw_template("class_impl.cs", CLASS_IMPL));
    tera_unwrap!(tera.add_raw_template("struct_impl.cs", STRUCT_IMPL));
    tera_unwrap!(tera.add_raw_template("union_impl.cs", UNION_IMPL));
    tera
}
