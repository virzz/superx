mod auth;
use auth::*;

use std::process::Command;

pub fn check_permission() -> bool {
    if cfg!(target_os = "macos") {
        macos::check_permission()
    } else {
        false
    }
}

pub fn install(target_name: &str) {
    if !cfg!(target_os = "macos") && !cfg!(target_os = "linux") {
        eprintln!("unsupported");
        std::process::exit(1);
    }

    let current_exe_str = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| std::process::exit(1));

    let script = format!(
        r#"
export SUPERX_TARGET_NAME="{}";
export SUPERX_CURRENT_EXE="{}";
export SUPERX_TARGET_PATH="/usr/local/sbin/$SUPERX_TARGET_NAME";
test -f "$SUPERX_TARGET_PATH" || exit 1;
test -f $SUPERX_CURRENT_EXE || exit 1;
if [ -f "$SUPERX_TARGET_PATH" ]; then
    chflags nouchg "$SUPERX_TARGET_PATH";
    chflags noschg "$SUPERX_TARGET_PATH";
fi
mkdir -p /usr/local/sbin/;
cp "$SUPERX_CURRENT_EXE" "$SUPERX_TARGET_PATH";
chown root:wheel "$SUPERX_TARGET_PATH";
chmod 4511 "$SUPERX_TARGET_PATH";
chflags uchg "$SUPERX_TARGET_PATH";
chflags schg "$SUPERX_TARGET_PATH";
test -x "$SUPERX_TARGET_PATH" || exit 1;
exit 0;
"#,
        target_name, current_exe_str
    );
    execute_command_with_env("sudo", &["sh"], &script);
}

fn execute_command_with_env(cmd: &str, args: &[&str], data: &str) {
    use std::io::Write;
    if let Ok(mut c) = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = c.stdin.take() {
            if let Ok(_) = stdin.write_all(data.as_bytes()) {
                if let Ok(_) = c.wait() {
                    return;
                }
            }
        }
    }
    std::process::exit(1);
}
