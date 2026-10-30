use super::*;
use alloc::borrow::ToOwned;
use std::cell::Cell;

#[test]
fn command_dispatch_preserves_install_and_shell_rules() {
    assert_eq!(cli::action(&[]), cli::Action::Shell);
    assert_eq!(cli::action(&[c"_install"]), cli::Action::Execute);
    assert_eq!(
        cli::action(&[c"_install", c"custom", c"ignored"]),
        cli::Action::Install(c"custom")
    );
    assert_eq!(cli::action(&[c"echo", c"a b", c""]), cli::Action::Execute);
}

#[test]
fn shell_prefers_bash_and_falls_back_to_sh_when_exec_returns() {
    let mut attempted = std::vec::Vec::new();
    cli::exec_shell_with(|command, args| {
        attempted.push(command.to_owned());
        assert_eq!(args, &[command, c"-i"]);
        // Returning simulates exec failure; successful exec never returns.
    });
    assert_eq!(attempted, [c"/bin/bash".to_owned(), c"/bin/sh".to_owned()]);
}

#[test]
fn authentication_only_prompts_for_password_when_needed() {
    let gui_calls = Cell::new(0);
    let password_calls = Cell::new(0);
    for (aqua, gui_ok, password_ok, expected, gui_count, password_count) in [
        (true, true, false, true, 1, 0),
        (true, false, true, true, 1, 1),
        (true, false, false, false, 1, 1),
        (false, true, true, true, 0, 1),
        (false, true, false, false, 0, 1),
    ] {
        gui_calls.set(0);
        password_calls.set(0);
        assert_eq!(
            auth::check_with(
                aqua,
                || {
                    gui_calls.set(gui_calls.get() + 1);
                    gui_ok
                },
                || {
                    password_calls.set(password_calls.get() + 1);
                    password_ok
                },
            ),
            expected
        );
        assert_eq!(gui_calls.get(), gui_count);
        assert_eq!(password_calls.get(), password_count);
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn installation_script_is_unchanged() {
    assert_eq!(
        install::script("test-superx", "/tmp/superx executable"),
        include_str!("../tests/fixtures/install.sh")
    );
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn executable_path_matches_standard_library() {
    assert_eq!(
        install::current_exe().unwrap().as_bytes(),
        std::env::current_exe()
            .unwrap()
            .as_os_str()
            .as_encoded_bytes()
    );
}

#[test]
fn process_execution_preserves_arguments_and_exit_status() {
    let status = unix::run_with_input(
        c"sh",
        &[c"sh", c"-c", c"[ \"$1\" = 'a b' ] && [ -z \"$2\" ] || exit 23; read -r line; [ \"$line\" = 'payload' ] || exit 24; exit 7", c"sh", c"a b", c""],
        b"payload\n",
    ).unwrap();
    assert!(libc::WIFEXITED(status));
    assert_eq!(libc::WEXITSTATUS(status), 7);
}

#[test]
fn missing_program_is_a_spawn_error() {
    assert!(
        unix::run_with_input(c"/nonexistent/superx-test", &[c"superx-test"], b"test\n").is_err()
    );
}

#[test]
fn exec_child_fixture() {
    if std::env::var_os("SUPERX_TEST_EXEC").is_some() {
        unix::exec(
            c"sh",
            &[
                c"sh",
                c"-c",
                c"[ \"$1\" = 'a b' ] && [ -z \"$2\" ] && exit 7; exit 23",
                c"sh",
                c"a b",
                c"",
            ],
        );
        std::process::exit(24);
    }
}

#[test]
fn exec_replaces_process_and_preserves_arguments() {
    // Run in a dedicated subprocess, without forking the multithreaded harness.
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "tests::exec_child_fixture"])
        .env("SUPERX_TEST_EXEC", "1")
        .env("PATH", "/bin:/usr/bin")
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(7));
}

#[test]
fn child_receives_eof_and_large_script_without_truncation() {
    let payload = std::vec![b'x'; 256 * 1024];
    let status = unix::run_with_input(
        c"sh",
        &[c"sh", c"-c", c"[ \"$(wc -c | tr -d ' ')\" = 262144 ]"],
        &payload,
    )
    .unwrap();
    assert!(libc::WIFEXITED(status));
    assert_eq!(libc::WEXITSTATUS(status), 0);
}

#[cfg(target_os = "linux")]
#[test]
fn linux_permission_check_uses_setuid_flow() {
    assert!(check_permission());
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
#[test]
fn unsupported_authentication_fails_closed() {
    assert!(!check_permission());
}
