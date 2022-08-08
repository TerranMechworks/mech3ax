use crate::enums::Enum;
use crate::structs::Struct;
use crate::unions::Union;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeResolver {
    enums: HashMap<String, Enum>,
    structs: HashMap<String, Struct>,
    unions: HashMap<String, Union>,
}

impl TypeResolver {
    pub fn new() -> Self {
        Self {
            enums: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
        }
    }

    pub fn push_enum<E>(&mut self)
    where
        E: mech3ax_metadata_types::Enum,
    {
        let c = Enum::new::<E>();
        self.enums.insert(c.name.to_string(), c);
    }

    pub fn push_struct<S>(&mut self)
    where
        S: mech3ax_metadata_types::Struct,
    {
        let s = Struct::new::<S>(self);
        self.structs.insert(s.name.to_string(), s);
    }

    pub fn push_union<S>(&mut self)
    where
        S: mech3ax_metadata_types::Union,
    {
        let s = Union::new::<S>(self);
        self.unions.insert(s.name.to_string(), s);
    }

    pub fn lookup_enum<'a, 'b>(&'a self, name: &'b str) -> Option<&Enum> {
        self.enums.get(name)
    }

    pub fn lookup_struct<'a, 'b>(&'a self, name: &'b str) -> Option<&Struct> {
        self.structs.get(name)
    }

    pub fn lookup_union<'a, 'b>(&'a self, name: &'b str) -> Option<&Union> {
        self.unions.get(name)
    }

    pub fn into_values(self) -> (Vec<Enum>, Vec<Struct>, Vec<Union>) {
        (
            self.enums.into_values().collect(),
            self.structs.into_values().collect(),
            self.unions.into_values().collect(),
        )
    }
}
