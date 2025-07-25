mod csharp_type;
mod enums;
mod fields;
mod flags;
mod module_path;
mod resolver;
mod structs;
mod templates;
mod unions;

pub(crate) use resolver::TypeResolver;

macro_rules! write {
    ($path:expr, $contents:ident) => {
        std::fs::write(&$path, $contents)
            .unwrap_or_else(|e| panic!("failed to write `{}`: {:?}", $path.display(), e));
    };
}

pub(crate) fn write(resolver: TypeResolver) {
    let resolver::TypeResolverValues {
        enums,
        structs,
        unions,
        flags,
        directories,
    } = resolver.into_values();

    for path in directories {
        std::fs::create_dir(&path)
            .unwrap_or_else(|e| panic!("failed to create `{}`: {:?}", path.display(), e));
    }

    let env = templates::make_env();

    for item in enums {
        let contents = item.render_impl(&env).unwrap();
        write!(item.path, contents);
    }

    for item in flags {
        let contents = item.render_impl(&env).unwrap();
        write!(item.path, contents);
    }

    for item in structs {
        let contents = item.render_impl(&env).unwrap();
        write!(item.path, contents);
    }

    for item in unions {
        let contents = item.render_impl(&env).unwrap();
        write!(item.path, contents);
    }
}
