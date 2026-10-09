use alloc::{ffi::CString, string::String, vec};
#[cfg(target_os = "macos")]
use core::ffi::CStr;

pub(crate) fn script(target_name: &str, executable: &str) -> String {
    let mut script = String::from("\nexport SUPERX_TARGET_NAME=\"");
    script.push_str(target_name);
    script.push_str("\";\nexport SUPERX_CURRENT_EXE=\"");
    script.push_str(executable);
    script.push_str("\";\n");
    script.push_str(include_str!("install.sh"));
    script
}

pub(crate) fn current_exe() -> Option<CString> {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn _NSGetExecutablePath(buffer: *mut libc::c_char, size: *mut u32) -> i32;
        }
        let mut size = 0;
        // SAFETY: size query uses a null buffer with zero capacity.
        unsafe { _NSGetExecutablePath(core::ptr::null_mut(), &mut size) };
        let mut buffer = vec![0u8; size as usize];
        // SAFETY: buffer has the size reported by dyld.
        if unsafe { _NSGetExecutablePath(buffer.as_mut_ptr().cast(), &mut size) } != 0 {
            return None;
        }
        // Match std::env::current_exe's canonicalized macOS path.
        // SAFETY: dyld supplied a NUL-terminated path; realpath allocates its result.
        unsafe {
            let path = libc::realpath(buffer.as_ptr().cast(), core::ptr::null_mut());
            if path.is_null() {
                return None;
            }
            let result = CStr::from_ptr(path).into();
            libc::free(path.cast());
            Some(result)
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut buffer = vec![0u8; 256];
        loop {
            // SAFETY: readlink writes at most buffer.len() bytes, without a NUL.
            let len = unsafe {
                libc::readlink(
                    c"/proc/self/exe".as_ptr(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                )
            };
            if len < 0 {
                return None;
            }
            if (len as usize) < buffer.len() {
                buffer.truncate(len as usize);
                return CString::new(buffer).ok();
            }
            buffer.resize(buffer.len().checked_mul(2)?, 0);
        }
    }
}

pub(crate) fn install(target_name: &str) {
    let result = current_exe().and_then(|path| {
        let path = path.to_str().ok()?;
        let script = script(target_name, path);
        crate::unix::run_with_input(c"sudo", &[c"sudo", c"sh"], script.as_bytes()).ok()
    });
    if result.is_none() {
        // SAFETY: terminate with the same installer failure status as before.
        unsafe { libc::exit(1) }
    }
    // As before, a successfully waited child counts as success regardless of status.
}
