use crate::enums::Enum;
use crate::options::Options;
use crate::structs::Struct;
use crate::unions::Union;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct TypeResolver {
    names: HashSet<&'static str>,
    enums: HashMap<&'static str, Enum>,
    structs: HashMap<&'static str, Struct>,
    unions: HashMap<&'static str, Union>,
    factory_converters: Vec<(String, usize)>,
}

impl TypeResolver {
    pub fn new() -> Self {
        Self {
            names: HashSet::new(),
            enums: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            factory_converters: Vec::new(),
        }
    }

    pub fn push_factory_converter(&mut self, converter: String, count: usize) {
        self.factory_converters.push((converter, count));
    }

    pub fn push_enum<E>(&mut self)
    where
        E: mech3ax_metadata_types::Enum,
    {
        let c = Enum::new::<E>();
        if !self.names.insert(c.name) {
            panic!("Duplicate type name `{}`", c.name);
        }
        self.enums.insert(c.name, c);
    }

    pub fn push_struct<S>(&mut self)
    where
        S: mech3ax_metadata_types::Struct,
    {
        let s = Struct::new::<S>(self);
        if !self.names.insert(s.name) {
            panic!("Duplicate type name `{}`", s.name);
        }
        self.structs.insert(s.name, s);
    }

    pub fn push_union<U>(&mut self)
    where
        U: mech3ax_metadata_types::Union,
    {
        let u = Union::new::<U>(self);
        if !self.names.insert(u.name) {
            panic!("Duplicate type name `{}`", u.name);
        }
        self.unions.insert(u.name, u);
    }

    pub fn lookup_enum<'a, 'b>(&'a self, name: &'b str) -> Option<&'a Enum> {
        self.enums.get(name)
    }

    pub fn lookup_struct<'a, 'b>(&'a self, name: &'b str) -> Option<&'a Struct> {
        self.structs.get(name)
    }

    pub fn lookup_union<'a, 'b>(&'a self, name: &'b str) -> Option<&'a Union> {
        self.unions.get(name)
    }

    pub fn into_values(self) -> (Vec<Enum>, Vec<Struct>, Vec<Union>, Options) {
        let mut factory_converters = self.factory_converters;
        factory_converters.sort();
        (
            self.enums.into_values().collect(),
            self.structs.into_values().collect(),
            self.unions.into_values().collect(),
            Options::new(factory_converters),
        )
    }
}
