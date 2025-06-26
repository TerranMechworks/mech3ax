mod enums;
mod fields;
mod flags;
mod module_path;
mod python_type;
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

const INIT_PY: &str = "from __future__ import annotations\n\n";

pub(crate) fn write(resolver: TypeResolver) {
    let resolver::TypeResolverValues {
        enums,
        flags,
        structs,
        unions,
        directories,
    } = resolver.into_values();

    for mut path in directories {
        std::fs::create_dir(&path)
            .unwrap_or_else(|e| panic!("failed to create `{}`: {:?}", path.display(), e));

        path.push("__init__.py");
        write!(path, INIT_PY);
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
