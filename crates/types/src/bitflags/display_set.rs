use std::fmt;

/// A debug set implementation that prints entries in display mode and in a single line
#[must_use = "must eventually call `finish()`"]
pub(crate) struct DisplaySet<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

impl<'a, 'b: 'a> DisplaySet<'a, 'b> {
    pub fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        let result = fmt.write_str("{");
        Self {
            fmt,
            result,
            has_fields: false,
        }
    }

    pub fn entry(&mut self, entry: &dyn fmt::Display) {
        self.result = self.result.and_then(|()| {
            if self.has_fields {
                self.fmt.write_str(", ")?;
            }
            entry.fmt(self.fmt)
        });
        self.has_fields = true;
    }

    pub fn finish(&mut self) -> fmt::Result {
        self.result.and_then(|()| self.fmt.write_str("}"))
    }
}
