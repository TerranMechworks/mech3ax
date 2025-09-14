use super::Ascii;
use crate::ConversionError;

pub fn node_name<const N: usize>(value: &Ascii<N>) -> Result<String, String> {
    value.to_str_node_name().map_err(|e| match e {
        ConversionError::PaddingError(padding) => format!("expected string padding `{}`", padding),
        ConversionError::NonAscii(index) => {
            format!("invalid character at +{}", index)
        }
        ConversionError::Unterminated => "missing zero terminator".to_string(),
    })
}

pub fn suffix<const N: usize>(value: &Ascii<N>) -> Result<String, String> {
    value.to_str_suffix().map_err(|e| match e {
        ConversionError::PaddingError(padding) => format!("expected string padding `{}`", padding),
        ConversionError::NonAscii(index) => {
            format!("invalid character at +{}", index)
        }
        ConversionError::Unterminated => "missing zero terminator".to_string(),
    })
}

pub fn garbage<const N: usize>(value: &Ascii<N>) -> Result<(String, Vec<u8>), String> {
    value.to_str_garbage().map_err(|e| match e {
        ConversionError::PaddingError(padding) => {
            // not possible
            format!("expected string padding `{}`", padding)
        }
        ConversionError::NonAscii(index) => {
            format!("invalid character at +{}", index)
        }
        ConversionError::Unterminated => "missing zero terminator".to_string(),
    })
}
