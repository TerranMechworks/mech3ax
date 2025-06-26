pub(crate) trait Resolver {
    fn push<TI>(&mut self)
    where
        TI: mech3ax_metadata_types::DerivedMetadata;
}

#[derive(Debug)]
struct ResolveErrorInner {
    path: Vec<&'static str>,
    name: &'static str,
    module_path: &'static str,
}

#[derive(Debug)]
pub(crate) struct ResolveError(Box<ResolveErrorInner>);

impl ResolveError {
    const DELIM: &'static str = ".";

    pub(crate) fn new(module_path: &'static str, name: &'static str) -> Self {
        let inner = ResolveErrorInner {
            path: vec![name, Self::DELIM],
            name,
            module_path,
        };
        Self(Box::new(inner))
    }

    pub(crate) fn push(mut self, name: &'static str) -> Self {
        self.0.path.push(name);
        self.0.path.push(Self::DELIM);
        self
    }

    pub(crate) fn into_string(self) -> String {
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
