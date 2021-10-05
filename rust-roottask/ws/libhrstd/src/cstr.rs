//! Abstractions over C-Strings. CString and CStr are part of Rust `std` but not
//! `core`, therefore I have to provide a custom abstraction.

use core::str::Utf8Error;

/// Possible errors when constructing a [`CStr`].
#[derive(Debug, PartialEq)]
pub enum CStrError {
    NotNullTerminated,
    Utf8(Utf8Error),
}

/// Small C-String wrapper around slices of `u8`. Expects that all bytes are valid ASCII/UTF-8.
/// All strings must be Null-Terminated.
///
/// This struct only helps to interpret memory as a CString. It is not possible to construct
/// C-Strings in memory.
///
/// This is what like `str` is to `String`.
#[derive(Debug)]
pub struct CStr<'a> {
    data: &'a [u8],
    str: &'a str,
    len: u32,
}

impl<'a> CStr<'a> {
    /// Constructs a C-String from the slice. It stops at the first null byte.
    pub fn new(data: &'a [u8]) -> Result<Self, CStrError> {
        let null_byte_index = data
            .iter()
            .enumerate()
            .find(|(_index, byte)| **byte == 0)
            .map(|(index, _)| index)
            .ok_or(CStrError::NotNullTerminated)?;
        let len = null_byte_index as u32;
        let str =
            core::str::from_utf8(&data[0..null_byte_index]).map_err(|e| CStrError::Utf8(e))?;
        Ok(Self { data, str, len })
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    /// Returns the length of the string without terminating NULL-byte.
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn as_str(&self) -> &'a str {
        self.str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr() {
        let data = b"foobar";
        assert_eq!(
            CStr::new(data).unwrap_err(),
            CStrError::NotNullTerminated,
            "must be null-terminated"
        );

        let data = b"foobar\0";
        assert_eq!(CStr::new(data).unwrap().len(), 6);
        assert_eq!(CStr::new(data).unwrap().str, "foobar");

        let data = b"foobar\0afafaf";
        assert_eq!(CStr::new(data).unwrap().len(), 6);
        assert_eq!(CStr::new(data).unwrap().str, "foobar");

        let data = b"\0";
        assert_eq!(CStr::new(data).unwrap().len(), 0);
        assert_eq!(CStr::new(data).unwrap().str, "");
    }
}
