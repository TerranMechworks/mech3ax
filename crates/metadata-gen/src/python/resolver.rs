use super::enums::Enum;
use super::flags::Flags;
use super::module_path::{path_mod_root, path_mod_types};
use super::python_type::PythonType;
use super::structs::Struct;
use super::unions::Union;
use crate::resolver::ResolveError;
use mech3ax_metadata_types::{
    TypeInfo, TypeInfoBase, TypeInfoEnum, TypeInfoFlags, TypeInfoOption, TypeInfoStruct,
    TypeInfoUnion, TypeInfoVec,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

type ResolveResult = ::std::result::Result<PythonType, ResolveError>;

#[derive(Debug)]
pub(crate) struct TypeResolver {
    enums: HashMap<(&'static str, &'static str), Enum>,
    flags: HashMap<(&'static str, &'static str), Flags>,
    structs: HashMap<(&'static str, &'static str), Struct>,
    unions: HashMap<(&'static str, &'static str), Union>,
    directories: HashSet<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct TypeResolverValues {
    pub(crate) enums: Vec<Enum>,
    pub(crate) flags: Vec<Flags>,
    pub(crate) structs: Vec<Struct>,
    pub(crate) unions: Vec<Union>,
    pub(crate) directories: Vec<PathBuf>,
}

impl crate::resolver::Resolver for TypeResolver {
    fn push<TI>(&mut self)
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
            TypeInfo::Flags(fi) => self.push_flags(fi),
        }
    }
}

impl TypeResolver {
    pub(crate) fn new() -> Self {
        let mut directories = HashSet::new();
        directories.insert(path_mod_root());
        directories.insert(path_mod_types());
        Self {
            enums: HashMap::new(),
            flags: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            directories,
        }
    }

    fn push_enum(&mut self, ei: &TypeInfoEnum) {
        let e = Enum::new(self, ei);
        self.enums.insert((ei.module_path, ei.name), e);
    }

    fn push_flags(&mut self, fi: &TypeInfoFlags) {
        let f = Flags::new(self, fi);
        self.flags.insert((fi.module_path, fi.name), f);
    }

    fn push_struct(&mut self, si: &TypeInfoStruct) {
        let s = Struct::new(self, si);
        self.structs.insert((si.module_path, si.name), s);
    }

    fn push_union(&mut self, ui: &TypeInfoUnion) {
        let u = Union::new(self, ui);
        self.unions.insert((ui.module_path, ui.name), u);
    }

    pub(crate) fn add_directory(&mut self, path: &Path) {
        self.directories.insert(path.to_path_buf());
    }

    pub(crate) fn resolve(&self, ti: &TypeInfo, name: &'static str) -> PythonType {
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
            TypeInfo::Flags(fi) => self.resolve_flags(fi),
        }
    }

    fn resolve_base(&self, bi: &TypeInfoBase) -> ResolveResult {
        // base types are cheap to resolve (being leaves)
        Ok(bi.into())
    }

    fn resolve_vec(&self, vi: &TypeInfoVec) -> ResolveResult {
        match self.resolve_inner(vi.inner) {
            // remap byte vec
            Ok(inner) if inner.is_byte() => Ok(PythonType::byte_vec()),
            Ok(inner) => Ok(PythonType::vec(inner)),
            Err(e) => Err(e.push("Vec")),
        }
    }

    fn resolve_option(&self, oi: &TypeInfoOption) -> ResolveResult {
        match self.resolve_inner(oi.inner) {
            Ok(inner) => Ok(PythonType::option(inner)),
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

    fn resolve_flags(&self, fi: &TypeInfoFlags) -> ResolveResult {
        // flags must be pushed before they can be resolved
        self.flags
            .get(&(fi.module_path, fi.name))
            .map(Flags::make_type)
            .ok_or_else(|| ResolveError::new(fi.module_path, fi.name))
    }

    fn resolve_struct(&self, si: &TypeInfoStruct) -> ResolveResult {
        // structs must be pushed before they can be resolved
        self.structs
            .get(&(si.module_path, si.name))
            .map(Struct::make_type)
            .ok_or_else(|| ResolveError::new(si.module_path, si.name))
    }

    fn resolve_union(&self, ui: &TypeInfoUnion) -> ResolveResult {
        // unions must be pushed before they can be resolved
        self.unions
            .get(&(ui.module_path, ui.name))
            .map(Union::make_type)
            .ok_or_else(|| ResolveError::new(ui.module_path, ui.name))
    }

    pub(crate) fn into_values(self) -> TypeResolverValues {
        let Self {
            enums,
            flags,
            structs,
            unions,
            directories,
        } = self;
        let mut directories: Vec<PathBuf> = directories.into_iter().collect();
        directories.sort();
        TypeResolverValues {
            enums: enums.into_values().collect(),
            flags: flags.into_values().collect(),
            structs: structs.into_values().collect(),
            unions: unions.into_values().collect(),
            directories,
        }
    }
}
