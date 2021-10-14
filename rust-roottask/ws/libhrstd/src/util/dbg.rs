//! Exports macro [`dbg`].

/// Convenient dbg-macro similar to the one from the standard library.
/// This forwards information to `log::debug!()`.
#[macro_export]
macro_rules! dbg {
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                #[cfg(not(test))]
                log::debug!("{} = {:#?}",stringify!($val), &tmp);
                #[cfg(test)]
                println!("{} = {:#?}",stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($self::dbg!($val)),+,)
    };
}
