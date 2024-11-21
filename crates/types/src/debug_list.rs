use std::fmt;

/// A debug list implementation that always prints entries without newlines,
/// regardless of the alternate/pretty formatter settings.
#[must_use = "must eventually call `finish()`"]
pub struct DebugList<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

impl<'a, 'b: 'a> DebugList<'a, 'b> {
    pub fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        let result = fmt.write_str("[");
        Self {
            fmt,
            result,
            has_fields: false,
        }
    }

    fn entry(&mut self, entry: &dyn fmt::Debug) {
        self.result = self.result.and_then(|()| {
            if self.has_fields {
                self.fmt.write_str(", ")?;
            }
            entry.fmt(self.fmt)
        });
        self.has_fields = true;
    }

    pub fn entries<D, I>(&mut self, entries: I) -> &mut Self
    where
        D: fmt::Debug,
        I: IntoIterator<Item = D>,
    {
        for entry in entries {
            self.entry(&entry);
        }
        self
    }

    pub fn finish(&mut self) -> fmt::Result {
        self.result.and_then(|()| self.fmt.write_str("]"))
    }
}
