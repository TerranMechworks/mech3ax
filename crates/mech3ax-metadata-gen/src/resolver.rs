use crate::csharp_type::CSharpType;
use crate::enums::Enum;
use crate::options::Options;
use crate::structs::Struct;
use crate::unions::Union;
use mech3ax_metadata_types::{
    TypeInfo, TypeInfoBase, TypeInfoEnum, TypeInfoOption, TypeInfoStruct, TypeInfoUnion,
    TypeInfoVec,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct TypeResolver {
    names: HashSet<&'static str>,
    enums: HashMap<&'static str, Enum>,
    structs: HashMap<&'static str, Struct>,
    unions: HashMap<&'static str, Union>,
    factory_converters: Vec<(String, usize)>,
}

#[derive(Debug)]
struct ResolveError(Vec<&'static str>);

impl ResolveError {
    const DELIM: &'static str = ".";

    pub fn new(name: &'static str) -> Self {
        Self(vec![name, Self::DELIM])
    }

    pub fn push(mut self, name: &'static str) -> Self {
        self.0.push(name);
        self.0.push(Self::DELIM);
        self
    }

    pub fn to_string(mut self) -> String {
        self.0.pop(); // remove last delimiter
        self.0.reverse();
        self.0.into_iter().collect()
    }
}

type ResolveResult = ::std::result::Result<CSharpType, ResolveError>;

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

    pub fn push<TI>(&mut self)
    where
        TI: mech3ax_metadata_types::DerivedMetadata,
    {
        match TI::TYPE_INFO {
            TypeInfo::Base(bi) => panic!("cannot push base type: {:?}", bi),
            TypeInfo::Vec(vi) => panic!("cannot push vec type: {:?}", vi),
            TypeInfo::Option(oi) => panic!("cannot push option type: {:?}", oi),
            TypeInfo::Enum(ei) => self.push_enum(ei),
            TypeInfo::Struct(si) => self.push_struct(si),
            TypeInfo::Union(ui) => self.push_union(ui),
        }
    }

    fn push_enum(&mut self, ei: &TypeInfoEnum) {
        let e = Enum::new(self, ei);
        if !self.names.insert(e.name) {
            panic!("Duplicate type name `{}`", e.name);
        }
        self.enums.insert(e.name, e);
    }

    fn push_struct(&mut self, si: &TypeInfoStruct) {
        let s = Struct::new(self, si);
        if !self.names.insert(s.name) {
            panic!("Duplicate type name `{}`", s.name);
        }
        self.structs.insert(s.name, s);
    }

    fn push_union(&mut self, ui: &TypeInfoUnion) {
        let u = Union::new(self, ui);
        if !self.names.insert(u.name) {
            panic!("Duplicate type name `{}`", u.name);
        }
        self.unions.insert(u.name, u);
    }

    pub fn resolve<'a, 'b>(&'a self, ti: &'b TypeInfo, name: &'static str) -> CSharpType {
        self.resolve_inner(ti).unwrap_or_else(|e| {
            let msg = e.push(name).to_string();
            panic!("`{}` not found", msg);
        })
    }

    fn resolve_inner<'a, 'b>(&'a self, ti: &'b TypeInfo) -> ResolveResult {
        match ti {
            TypeInfo::Base(bi) => self.resolve_base(bi),
            TypeInfo::Enum(ei) => self.resolve_enum(ei),
            TypeInfo::Vec(vi) => self.resolve_vec(vi),
            TypeInfo::Option(oi) => self.resolve_option(oi),
            TypeInfo::Struct(si) => self.resolve_struct(si),
            TypeInfo::Union(ui) => self.resolve_union(ui),
        }
    }

    fn resolve_base<'a, 'b>(&'a self, bi: &'b TypeInfoBase) -> ResolveResult {
        // base types are cheap to resolve (being leaves)
        Ok(bi.into())
    }

    fn resolve_vec<'a, 'b>(&'a self, vi: &'b TypeInfoVec) -> ResolveResult {
        match self.resolve_inner(vi.inner) {
            // remap byte vec
            Ok(inner) if inner.is_byte() => Ok(CSharpType::byte_vec()),
            Ok(inner) => Ok(CSharpType::vec(inner)),
            Err(e) => Err(e.push("Vec")),
        }
    }

    fn resolve_option<'a, 'b>(&'a self, oi: &'b TypeInfoOption) -> ResolveResult {
        match self.resolve_inner(oi.inner) {
            Ok(inner) => Ok(CSharpType::option(inner)),
            Err(e) => Err(e.push("Option")),
        }
    }

    fn resolve_enum<'a, 'b>(&'a self, ei: &'b TypeInfoEnum) -> ResolveResult {
        // enums must be pushed before they can be resolved
        self.enums
            .get(ei.name)
            .map(Enum::make_type)
            .ok_or_else(|| ResolveError::new(ei.name))
    }

    fn resolve_struct<'a, 'b>(&'a self, si: &'b TypeInfoStruct) -> ResolveResult {
        // enums must be pushed before they can be resolved
        self.structs
            .get(si.name)
            .map(Struct::make_type)
            .ok_or_else(|| ResolveError::new(si.name))
    }

    fn resolve_union<'a, 'b>(&'a self, ui: &'b TypeInfoUnion) -> ResolveResult {
        // enums must be pushed before they can be resolved
        self.unions
            .get(ui.name)
            .map(Union::make_type)
            .ok_or_else(|| ResolveError::new(ui.name))
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
