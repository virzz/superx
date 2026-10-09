#[cfg(target_os = "macos")]
mod ffi;
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(any(target_os = "macos", all(test, target_os = "linux")))]
pub(crate) fn check_with(
    aqua: bool,
    graphical: impl FnOnce() -> bool,
    password: impl FnOnce() -> bool,
) -> bool {
    (aqua && graphical()) || password()
}
