#![cfg_attr(
    all(
        not(test),
        not(debug_assertions),
        any(target_os = "macos", target_os = "linux")
    ),
    no_std
)]
#![cfg_attr(
    all(
        not(test),
        not(debug_assertions),
        any(target_os = "macos", target_os = "linux")
    ),
    no_main
)]

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    any(test, not(debug_assertions))
))]
mod allocator;

#[cfg(all(
    not(test),
    not(debug_assertions),
    any(target_os = "macos", target_os = "linux")
))]
mod native {
    extern crate alloc;
    use crate::allocator::SystemAllocator;
    use alloc::vec::Vec;
    use core::ffi::CStr;

    // Without std, Linux's C runtime must be linked explicitly (including musl).
    #[cfg(target_os = "linux")]
    #[link(name = "c")]
    unsafe extern "C" {}

    #[global_allocator]
    static ALLOCATOR: SystemAllocator = SystemAllocator;

    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
        // Release already used panic=abort. Omit std's backtrace/formatting runtime.
        unsafe { libc::abort() }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn main(argc: libc::c_int, argv: *const *const libc::c_char) -> libc::c_int {
        // Match std's handling of closed standard descriptors before auth opens
        // any files. The lowest available descriptor is filled on each iteration.
        for fd in 0..3 {
            // SAFETY: fcntl queries fd; open uses a static NUL-terminated path.
            unsafe {
                if libc::fcntl(fd, libc::F_GETFD) < 0
                    && libc::open(c"/dev/null".as_ptr(), libc::O_RDWR) < 0
                {
                    libc::abort();
                }
            }
        }
        // SAFETY: the platform C runtime supplies argc readable pointers to
        // NUL-terminated strings; they remain valid until main returns.
        let args: Vec<&CStr> = unsafe { core::slice::from_raw_parts(argv, argc as usize) }
            .iter()
            .skip(1)
            .map(|arg| unsafe { CStr::from_ptr(*arg) })
            .collect();
        // Match std startup: a broken installer pipe becomes an I/O failure.
        unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };
        superx::cli::run(&args)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 1 && args[0] == "_install" {
        superx::install(&args[1]);
    }
    // Platforms without an authentication backend fail closed.
    std::process::exit(1);
}

#[cfg(all(debug_assertions, any(target_os = "macos", target_os = "linux")))]
fn main() {
    // Keep std diagnostics in development; only release needs the tiny runtime.
    let args: Vec<std::ffi::CString> = std::env::args()
        .skip(1)
        .map(|arg| std::ffi::CString::new(arg).expect("argv cannot contain NUL"))
        .collect();
    let args: Vec<&std::ffi::CStr> = args.iter().map(|arg| arg.as_c_str()).collect();
    std::process::exit(superx::cli::run(&args));
}
