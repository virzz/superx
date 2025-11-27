use std::os::unix::process::CommandExt;
use std::process::Command;

use nix::unistd::{Gid, ROOT, geteuid, setegid, setuid};

use superx::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 1 && args[0] == "_install" {
        install(&args[1]);
        std::process::exit(0);
    }

    if !check_permission() {
        std::process::exit(1);
    }

    // 检查并设置 root 权限
    if !geteuid().is_root() || setuid(ROOT).is_err() || setegid(Gid::from_raw(0)).is_err() {
        std::process::exit(1);
    };

    // 执行命令
    if args.is_empty() {
        Command::new("/bin/sh").arg("-i").exec()
    } else {
        Command::new(&args[0]).args(&args[1..]).exec()
    };
}
