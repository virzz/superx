#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

struct Fixture(PathBuf);
impl Fixture {
    fn new(name: &str, script: Option<&str>) -> Self {
        let path = std::env::temp_dir().join(format!("superx-{}-{name}", std::process::id()));
        fs::create_dir(&path).unwrap();
        if let Some(script) = script {
            let sudo = path.join("sudo");
            fs::write(&sudo, script).unwrap();
            fs::set_permissions(&sudo, fs::Permissions::from_mode(0o700)).unwrap();
        }
        Self(path)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_superx"));
        command
            .env("PATH", &self.0)
            .env("SUPERX_TEST_CAPTURE", self.0.join("captured"))
            .args(["_install", "custom target"]);
        command
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn installer_passes_original_script_to_sudo_sh_and_preserves_status() {
    // No sudo or installation is performed: the isolated PATH supplies a recorder.
    let fixture = Fixture::new(
        "capture",
        Some(
            "#!/bin/sh\n[ \"$#\" = 1 ] && [ \"$1\" = sh ] || exit 99\n/bin/cat > \"$SUPERX_TEST_CAPTURE\"\nexit 7\n",
        ),
    );
    assert_eq!(fixture.command().status().unwrap().code(), Some(0));
    let captured = fs::read_to_string(fixture.0.join("captured")).unwrap();
    let executable = fs::canonicalize(env!("CARGO_BIN_EXE_superx")).unwrap();
    let expected = include_str!("fixtures/install.sh")
        .replace("test-superx", "custom target")
        .replace("/tmp/superx executable", executable.to_str().unwrap());
    assert_eq!(captured, expected);
}

#[test]
fn installer_reports_missing_sudo() {
    let fixture = Fixture::new("missing", None);
    assert_eq!(fixture.command().status().unwrap().code(), Some(1));
}

#[test]
fn installer_handles_closed_standard_descriptors() {
    use std::os::unix::process::CommandExt;
    let fixture = Fixture::new(
        "closed-fds",
        Some("#!/bin/sh\n/bin/cat > \"$SUPERX_TEST_CAPTURE\"\n"),
    );
    let mut command = fixture.command();
    // SAFETY: the child callback uses only async-signal-safe close, without locks.
    unsafe {
        command.pre_exec(|| {
            for fd in 0..3 {
                libc::close(fd);
            }
            Ok(())
        });
    }
    assert_eq!(command.status().unwrap().code(), Some(0));
    assert!(fs::read(fixture.0.join("captured")).unwrap().len() > 100);
}
