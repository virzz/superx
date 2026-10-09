fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg-bin=superx=-Wl,-dead_strip_dylibs");
        if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64") {
            // Chained fixups are supported since macOS 11, also the minimum OS
            // for Apple Silicon. They remove the legacy lazy-binding DATA page.
            // Do not raise the deployment target of the Intel build.
            println!("cargo:rustc-link-arg-bin=superx=-Wl,-fixup_chains");
        }
    }
}
