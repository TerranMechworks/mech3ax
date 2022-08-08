use crate::enums::enum_macro;
use crate::structs::struct_macro;
use crate::unions::union_macro;
use std::error::Error;
use tera::Tera;

macro_rules! tera_unwrap {
    ($stmt:expr) => {{
        if let Err(e) = $stmt {
            if let Some(source) = e.source() {
                eprintln!("{}", source);
            }
            panic!("{}", e);
        }
    }};
}

const MACROS: &'static str = concat!(enum_macro!(), struct_macro!(), union_macro!());

const ENUM_TEMPLATE: &'static str = r###"{% import "macros.cs" as macros -%}
using System;
using Mech3DotNet.Json.Converters;
using Newtonsoft.Json;
using static Mech3DotNet.Json.Converters.Helpers;

namespace Mech3DotNet.Json
{
{{ macros::make_enum(enum=enum) }}
}
"###;

const STRUCT_TEMPLATE: &'static str = r###"{% import "macros.cs" as macros -%}
using System;
using System.Collections.Generic;
using Mech3DotNet.Json.Converters;
using Newtonsoft.Json;
using static Mech3DotNet.Json.Converters.Helpers;

namespace Mech3DotNet.Json
{
{{ macros::make_struct(struct=struct) }}
}
"###;

const UNION_TEMPLATE: &'static str = r###"{% import "macros.cs" as macros -%}
using System;
using Mech3DotNet.Json.Converters;
using Newtonsoft.Json;
using static Mech3DotNet.Json.Converters.Helpers;

namespace Mech3DotNet.Json
{
{{ macros::make_union(union=union) }}
}
"###;

pub fn make_tera() -> Tera {
    let mut tera = Tera::default();
    tera_unwrap!(tera.add_raw_template("macros.cs", MACROS));
    tera_unwrap!(tera.add_raw_template("enum.cs", ENUM_TEMPLATE));
    tera_unwrap!(tera.add_raw_template("struct.cs", STRUCT_TEMPLATE));
    tera_unwrap!(tera.add_raw_template("union.cs", UNION_TEMPLATE));
    tera
}
