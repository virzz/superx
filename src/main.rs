use nix::unistd::{Gid, ROOT, geteuid, setegid, setuid};
use std::os::unix::process::CommandExt;
use std::process::Command;

// sudo chown root:root {}
// sudo chown root:wheel {}
// sudo chmod 4511 {}
// sudo chflags uchg {} User不可改
// sudo chflags schg {} SuperUser不可改

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !geteuid().is_root() || setuid(ROOT).is_err() || setegid(Gid::from_raw(0)).is_err() {
        println!("no root");
        std::process::exit(1);
    };
    let err = if args.is_empty() {
        Command::new("/bin/sh").arg("-i").exec()
    } else {
        Command::new(&args[0]).args(&args[1..]).exec()
    };
    eprintln!("执行失败: {}", err);
    std::process::exit(1);
}
