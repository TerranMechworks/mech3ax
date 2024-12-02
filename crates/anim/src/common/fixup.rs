use log::debug;
use mech3ax_common::assert::format_conversion_err;
use mech3ax_common::Result;
use mech3ax_types::{Ascii, ConversionError};

pub(crate) struct Fwd<F, const N: usize>
where
    F: Fn(&[u8; N]) -> Option<(u32, &'static str)>,
{
    name: &'static str,
    f: F,
}

impl<F, const N: usize> Fwd<F, N>
where
    F: Fn(&[u8; N]) -> Option<(u32, &'static str)>,
{
    pub(crate) fn new(name: &'static str, f: F) -> Self {
        Self { name, f }
    }

    pub(crate) fn fixup(&self, pos: usize, value: &Ascii<N>) -> Result<(String, Option<u32>)> {
        value
            .to_str_padded()
            .map(|v| (v, None))
            .or_else(|e| match &e {
                ConversionError::PaddingError(_) => (self.f)(value.as_ref())
                    .map(|(hash, v)| {
                        debug!("{} fixup: `{}` -> `{}`", self.name, value.escape_ascii(), v);
                        (v.to_string(), Some(hash))
                    })
                    .ok_or_else(|| format_conversion_err(self.name, pos, e).into()),
                _ => Err(format_conversion_err(self.name, pos, e).into()),
            })
    }
}

pub(crate) struct Rev<F, const N: usize>
where
    F: Fn(u32, &str) -> Option<&'static [u8; N]>,
{
    name: &'static str,
    f: F,
}

impl<F, const N: usize> Rev<F, N>
where
    F: Fn(u32, &str) -> Option<&'static [u8; N]>,
{
    pub(crate) fn new(name: &'static str, f: F) -> Self {
        Self { name, f }
    }

    pub(crate) fn fixup(&self, value: &str, hash: Option<u32>) -> Ascii<N> {
        hash.and_then(|hash| (self.f)(hash, value))
            .map(|bytes| {
                let v = Ascii::new(bytes);
                debug!("{} fixup: `{}` <- `{}`", self.name, v.escape_ascii(), value,);
                v
            })
            .unwrap_or_else(|| Ascii::from_str_padded(value))
    }
}
