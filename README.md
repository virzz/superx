# SuperX

SuperX is a tool for macOS to authenticate users with Touch ID or password.

## Install

```bash
sudo chown root:root superx
sudo chown root:wheel superx
sudo chmod 4511 superx
sudo chflags uchg superx # User不可改
sudo chflags schg superx # SuperUser不可改
```

## Usage

```bash
./superx
```

## Build and binary size

```bash
cargo build --release --locked
cargo test --locked
cargo test --release --locked
```

macOS and Linux release binaries use `no_std + alloc` with the platform C
runtime, a libc allocator, and direct process/authorization APIs. Debug builds
retain the standard Rust runtime and diagnostics. No nightly toolchain or
post-build packing/stripping is required. Release panics still abort, but no
longer print Rust panic diagnostics or backtraces.

On macOS ARM64, `build.rs` enables chained fixups (supported by the platform's
existing macOS 11 minimum) and strips unused dylib references. Intel macOS keeps
its existing deployment target. These linker flags are never passed to Linux
or Windows builds.

Measured with rustc 1.97.1 on macOS ARM64: the release executable decreased from
321,600 to 35,808 bytes (88.9%). `opt-level = "s"` was smaller than `"z"` in the
comparison. Exact sizes vary with toolchain and SDK versions.

Authentication remains macOS-only: Aqua graphical authentication falls back to
the existing terminal password flow. The `_install` entry point and embedded
installation script are retained. Non-macOS authentication fails closed; Linux
retains the existing installer, including its macOS-oriented shell commands.
Other platforms compile with an unsupported installer and exit with failure.

Validated targets: macOS ARM64/x86_64 and Linux musl ARM64/x86_64 release linking;
Windows MSVC ARM64/x86_64 compile checks. Cross-compiling Linux requires an
appropriate linker, for example:

```bash
cargo build --release --locked --target aarch64-unknown-linux-musl \
  --config 'target.aarch64-unknown-linux-musl.linker="aarch64-linux-musl-gcc"'
```

Tests cover authentication fallback, argument dispatch, process execution,
allocator alignment, and installation script delivery using a fake `sudo`.
They do not display an authentication dialog, elevate privileges, or install
the executable.

## License

MIT
