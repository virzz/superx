#![no_std]

extern crate alloc;
#[cfg(test)]
extern crate std;

mod auth;
pub mod cli;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod install;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod unix;

pub fn check_permission() -> bool {
    #[cfg(target_os = "macos")]
    return auth::macos::check_permission();
    #[cfg(not(target_os = "macos"))]
    false
}

pub fn install(target_name: &str) {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    install::install(target_name);
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = target_name;
        // This API historically terminated on unsupported installer platforms.
        extern crate std;
        std::eprintln!("unsupported");
        std::process::exit(1);
    }
}

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod tests;
