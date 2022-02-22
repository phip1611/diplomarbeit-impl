//! Linux constants.

/// Count of chars in a file name including null byte.
///
/// Source: <https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/limits.h#L13>
#[allow(unused)]
pub const LINUX_NAME_MAX: usize = 255;
/// Count of chars in a path name including null byte.
///
/// Source: <https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/limits.h#L13>
pub const LINUX_PATH_MAX: usize = 4096;
