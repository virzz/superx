use alloc::vec::Vec;
use core::{ffi::CStr, mem::MaybeUninit, ptr};

fn argv(args: &[&CStr]) -> Vec<*mut libc::c_char> {
    args.iter()
        .map(|arg| arg.as_ptr().cast_mut())
        .chain(core::iter::once(ptr::null_mut()))
        .collect()
}

fn errno() -> i32 {
    // SAFETY: libc returns a valid pointer to this thread's errno.
    unsafe {
        #[cfg(target_os = "macos")]
        return *libc::__error();
        #[cfg(target_os = "linux")]
        return *libc::__errno_location();
    }
}

pub(crate) fn exec(command: &CStr, args: &[&CStr]) {
    let args = argv(args);
    // SAFETY: all strings outlive the call and argv has a trailing null pointer.
    // Match Command's restoration of SIGPIPE before executing the target.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        libc::execvp(command.as_ptr(), args.as_ptr().cast());
    }
}

struct Fd(libc::c_int);
impl Drop for Fd {
    fn drop(&mut self) {
        // SAFETY: this guard exclusively owns an open descriptor.
        unsafe { libc::close(self.0) };
    }
}

/// Return the raw wait status, preserving the installer's historical treatment
/// of a nonzero child exit status as a completed installation attempt.
pub(crate) fn run_with_input(command: &CStr, args: &[&CStr], mut data: &[u8]) -> Result<i32, ()> {
    let args = argv(args);
    let mut pipe = [-1; 2];
    // SAFETY: pipe has space for both descriptors.
    if unsafe { libc::pipe(pipe.as_mut_ptr()) } != 0 {
        return Err(());
    }
    let reader = Fd(pipe[0]);
    let writer = Fd(pipe[1]);
    // Keep descriptors away from stdin/stdout/stderr, including when the caller
    // started with a closed standard descriptor. CLOEXEC prevents leakage.
    fn duplicate(fd: Fd) -> Result<Fd, ()> {
        // SAFETY: fd is open and F_DUPFD_CLOEXEC accepts a minimum descriptor.
        let result = unsafe { libc::fcntl(fd.0, libc::F_DUPFD_CLOEXEC, 3) };
        if result < 0 { Err(()) } else { Ok(Fd(result)) }
    }
    let reader = duplicate(reader)?;
    let writer = duplicate(writer)?;
    let mut actions = MaybeUninit::<libc::posix_spawn_file_actions_t>::uninit();
    let mut attr = MaybeUninit::<libc::posix_spawnattr_t>::uninit();
    let mut pid = 0;
    // SAFETY: each object is initialized before use and destroyed on every path.
    // All referenced strings, argv entries and environ remain live during spawn.
    let spawned = unsafe {
        if libc::posix_spawn_file_actions_init(actions.as_mut_ptr()) != 0 {
            return Err(());
        }
        let actions = actions.as_mut_ptr();
        if libc::posix_spawnattr_init(attr.as_mut_ptr()) != 0 {
            libc::posix_spawn_file_actions_destroy(actions);
            return Err(());
        }
        let attr = attr.as_mut_ptr();
        let mut signals = MaybeUninit::<libc::sigset_t>::uninit();
        libc::sigemptyset(signals.as_mut_ptr());
        libc::sigaddset(signals.as_mut_ptr(), libc::SIGPIPE);
        let configured = libc::posix_spawn_file_actions_adddup2(actions, reader.0, 0) == 0
            && libc::posix_spawn_file_actions_addclose(actions, reader.0) == 0
            && libc::posix_spawn_file_actions_addclose(actions, writer.0) == 0
            && libc::posix_spawnattr_setsigdefault(attr, signals.as_ptr()) == 0
            && libc::posix_spawnattr_setflags(attr, libc::POSIX_SPAWN_SETSIGDEF as _) == 0;
        #[cfg(target_os = "macos")]
        let env = {
            unsafe extern "C" {
                fn _NSGetEnviron() -> *mut *mut *mut libc::c_char;
            }
            *_NSGetEnviron()
        };
        #[cfg(target_os = "linux")]
        let env = {
            unsafe extern "C" {
                static mut environ: *mut *mut libc::c_char;
            }
            environ
        };
        let result = configured
            && libc::posix_spawnp(
                &mut pid,
                command.as_ptr(),
                actions,
                attr,
                args.as_ptr(),
                env,
            ) == 0;
        libc::posix_spawnattr_destroy(attr);
        libc::posix_spawn_file_actions_destroy(actions);
        result
    };
    drop(reader);
    if !spawned {
        return Err(());
    }
    let mut written = true;
    while !data.is_empty() {
        // SAFETY: data is a live byte slice, and writer owns the pipe descriptor.
        let n = unsafe { libc::write(writer.0, data.as_ptr().cast(), data.len()) };
        if n < 0 && errno() == libc::EINTR {
            continue;
        }
        if n <= 0 {
            written = false;
            break;
        }
        data = &data[n as usize..];
    }
    drop(writer);
    let mut status = 0;
    loop {
        // SAFETY: pid is our spawned child; status points to initialized storage.
        if unsafe { libc::waitpid(pid, &mut status, 0) } >= 0 {
            return if written { Ok(status) } else { Err(()) };
        }
        if errno() != libc::EINTR {
            return Err(());
        }
    }
}
