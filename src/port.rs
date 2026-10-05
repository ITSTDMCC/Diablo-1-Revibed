//! Porting support: loud failures for code paths that are not ported yet.

/// Stops the game at a function that has not been ported, naming it and its address in
/// devilutionx.exe so the next porting step is obvious.
#[macro_export]
macro_rules! unported {
    ($key:expr) => {
        panic!("unported function: {}", $key)
    };
    ($key:expr, $addr:expr) => {
        panic!("unported function: {} @ {:#x}", $key, $addr)
    };
}

/// Declares a function that is not ported yet: same signature as the eventual port, body fails
/// loudly with the manifest key. `// @pending` marks it for tools/stubs.py.
#[macro_export]
macro_rules! pending_fn {
    ($(#[$m:meta])* $vis:vis fn $name:ident($($arg:ident: $ty:ty),* $(,)?) $(-> $ret:ty)?, $key:literal) => {
        $(#[$m])*
        #[allow(unused_variables)]
        $vis fn $name($($arg: $ty),*) $(-> $ret)? {
            $crate::unported!($key)
        }
    };
}
