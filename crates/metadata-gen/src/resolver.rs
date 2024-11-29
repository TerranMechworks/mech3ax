use crate::csharp_type::CSharpType;
use crate::enums::Enum;
use crate::module_path::path_mod_root;
use crate::structs::Struct;
use crate::unions::Union;
use mech3ax_metadata_types::{
    TypeInfo, TypeInfoBase, TypeInfoEnum, TypeInfoOption, TypeInfoStruct, TypeInfoUnion,
    TypeInfoVec,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TypeResolver {
    names: HashSet<String>,
    enums: HashMap<(&'static str, &'static str), Enum>,
    structs: HashMap<(&'static str, &'static str), Struct>,
    unions: HashMap<(&'static str, &'static str), Union>,
    directories: HashSet<PathBuf>,
}

#[derive(Debug)]
struct ResolveErrorInner {
    path: Vec<&'static str>,
    name: &'static str,
    module_path: &'static str,
}

#[derive(Debug)]
struct ResolveError(Box<ResolveErrorInner>);

impl ResolveError {
    const DELIM: &'static str = ".";

    pub fn new(module_path: &'static str, name: &'static str) -> Self {
        let inner = ResolveErrorInner {
            path: vec![name, Self::DELIM],
            name,
            module_path,
        };
        Self(Box::new(inner))
    }

    pub fn push(mut self, name: &'static str) -> Self {
        self.0.path.push(name);
        self.0.path.push(Self::DELIM);
        self
    }

    pub fn into_string(self) -> String {
        let mut inner = self.0;
        inner.path.pop(); // remove last delimiter
        inner.path.reverse();
        let path: String = inner.path.into_iter().collect();
        format!(
            "type `{}::{}` required by `{}` not found",
            inner.module_path, inner.name, path
        )
    }
}

type ResolveResult = ::std::result::Result<CSharpType, ResolveError>;

impl TypeResolver {
    pub fn new() -> Self {
        let mut directories = HashSet::new();
        directories.insert(path_mod_root());
        Self {
            names: HashSet::new(),
            enums: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            directories,
        }
    }

    pub fn push<TI>(&mut self) -> String
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

    fn push_enum(&mut self, ei: &TypeInfoEnum) -> String {
        let e = Enum::new(self, ei);
        if !self.names.insert(e.full_name.clone()) {
            panic!("Duplicate type name `{}`", e.full_name);
        }
        let full_name = e.full_name.clone();
        self.enums.insert((ei.module_path, ei.name), e);
        full_name
    }

    fn push_struct(&mut self, si: &TypeInfoStruct) -> String {
        let s = Struct::new(self, si);
        if !self.names.insert(s.full_name.clone()) {
            panic!("Duplicate type name `{}`", s.full_name);
        }
        let full_name = s.full_name.clone();
        self.structs.insert((si.module_path, si.name), s);
        full_name
    }

    fn push_union(&mut self, ui: &TypeInfoUnion) -> String {
        let u = Union::new(self, ui);
        if !self.names.insert(u.full_name.clone()) {
            panic!("Duplicate type name `{}`", u.full_name);
        }
        let full_name = u.full_name.clone();
        self.unions.insert((ui.module_path, ui.name), u);
        full_name
    }

    pub fn add_directory(&mut self, path: &Path) {
        self.directories.insert(path.to_path_buf());
    }

    pub fn resolve(&self, ti: &TypeInfo, name: &'static str) -> CSharpType {
        self.resolve_inner(ti).unwrap_or_else(|e| {
            let msg = e.push(name).into_string();
            panic!("`{}` not found", msg);
        })
    }

    fn resolve_inner(&self, ti: &TypeInfo) -> ResolveResult {
        match ti {
            TypeInfo::Base(bi) => self.resolve_base(bi),
            TypeInfo::Enum(ei) => self.resolve_enum(ei),
            TypeInfo::Vec(vi) => self.resolve_vec(vi),
            TypeInfo::Option(oi) => self.resolve_option(oi),
            TypeInfo::Struct(si) => self.resolve_struct(si),
            TypeInfo::Union(ui) => self.resolve_union(ui),
        }
    }

    fn resolve_base(&self, bi: &TypeInfoBase) -> ResolveResult {
        // base types are cheap to resolve (being leaves)
        Ok(bi.into())
    }

    fn resolve_vec(&self, vi: &TypeInfoVec) -> ResolveResult {
        match self.resolve_inner(vi.inner) {
            // remap byte vec
            Ok(inner) if inner.is_byte() => Ok(CSharpType::byte_vec()),
            Ok(inner) => Ok(CSharpType::vec(inner)),
            Err(e) => Err(e.push("Vec")),
        }
    }

    fn resolve_option(&self, oi: &TypeInfoOption) -> ResolveResult {
        match self.resolve_inner(oi.inner) {
            Ok(inner) => Ok(CSharpType::option(inner)),
            Err(e) => Err(e.push("Option")),
        }
    }

    fn resolve_enum(&self, ei: &TypeInfoEnum) -> ResolveResult {
        // enums must be pushed before they can be resolved
        self.enums
            .get(&(ei.module_path, ei.name))
            .map(Enum::make_type)
            .ok_or_else(|| ResolveError::new(ei.module_path, ei.name))
    }

    fn resolve_struct(&self, si: &TypeInfoStruct) -> ResolveResult {
        // enums must be pushed before they can be resolved
        self.structs
            .get(&(si.module_path, si.name))
            .map(Struct::make_type)
            .ok_or_else(|| ResolveError::new(si.module_path, si.name))
    }

    fn resolve_union(&self, ui: &TypeInfoUnion) -> ResolveResult {
        // enums must be pushed before they can be resolved
        self.unions
            .get(&(ui.module_path, ui.name))
            .map(Union::make_type)
            .ok_or_else(|| ResolveError::new(ui.module_path, ui.name))
    }

    pub fn into_values(self) -> (Vec<Enum>, Vec<Struct>, Vec<Union>, Vec<PathBuf>) {
        let Self {
            names: _,
            enums,
            structs,
            unions,
            directories,
        } = self;
        (
            enums.into_values().collect(),
            structs.into_values().collect(),
            unions.into_values().collect(),
            directories.into_iter().collect(),
        )
    }
}
