use core::ffi::CStr;

#[derive(Debug, PartialEq, Eq)]
pub enum Action<'a> {
    Install(&'a CStr),
    Shell,
    Execute,
}

pub fn action<'a>(args: &[&'a CStr]) -> Action<'a> {
    match args {
        [] => Action::Shell,
        [command, name, ..] if *command == c"_install" => Action::Install(name),
        _ => Action::Execute,
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(crate) fn exec_shell_with(mut exec: impl FnMut(&CStr, &[&CStr])) {
    // A successful exec replaces this process; only failure reaches the fallback.
    exec(c"/bin/bash", &[c"/bin/bash", c"-i"]);
    exec(c"/bin/sh", &[c"/bin/sh", c"-i"]);
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn run(args: &[&CStr]) -> i32 {
    // std::env::args previously required Unicode for every argument.
    if args.iter().any(|arg| arg.to_str().is_err()) {
        // SAFETY: abort takes no arguments and never returns.
        unsafe { libc::abort() }
    }
    let action = action(args);
    if let Action::Install(name) = action {
        if let Ok(name) = name.to_str() {
            crate::install(name);
        }
        return 0;
    }
    if !crate::check_permission() {
        return 1;
    }
    // SAFETY: these calls take no pointers and retain the original ordering.
    unsafe {
        if libc::geteuid() != 0 || libc::setuid(0) != 0 || libc::setegid(0) != 0 {
            return 1;
        }
    }
    if action == Action::Shell {
        exec_shell_with(crate::unix::exec);
    } else if let Some(command) = args.first() {
        crate::unix::exec(command, args);
    }
    // The original main also returned success when exec failed.
    0
}
