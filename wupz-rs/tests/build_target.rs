#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cross_targets_never_invoke_the_native_zig_toolchain() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch = Scratch(
        std::env::temp_dir().join(format!("wupz-build-target-{}-{unique}", std::process::id())),
    );
    fs::create_dir(&scratch.0).unwrap();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = scratch.0.join("build-script");
    let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let compiled = Command::new(compiler)
        .arg("--edition=2024")
        .arg(manifest.join("build.rs"))
        .arg("-o")
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );

    // A tool trap catches an attempted host build even if it would later fall
    // back because Zig was missing or compilation failed on the CI machine.
    let zig = scratch.0.join("zig");
    fs::write(
        &zig,
        "#!/bin/sh\nprintf invoked > \"$WUPZ_TEST_MARKER\"\nexit 1\n",
    )
    .unwrap();
    fs::set_permissions(&zig, fs::Permissions::from_mode(0o700)).unwrap();
    let marker = scratch.0.join("invoked");
    for target in [
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
    ] {
        let out = scratch.0.join(target);
        fs::create_dir(&out).unwrap();
        let result = Command::new(&script)
            .env("CARGO_MANIFEST_DIR", &manifest)
            .env("OUT_DIR", &out)
            .env("HOST", "aarch64-apple-darwin")
            .env("TARGET", target)
            .env("PATH", &scratch.0)
            .env("WUPZ_TEST_MARKER", &marker)
            .env_remove("WUPZ_NO_ZIG")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(result.status.success(), "{target}: {stdout}");
        assert!(
            !marker.exists(),
            "{target}: tried to run the host Zig toolchain"
        );
        assert!(
            !stdout.contains("cargo:rustc-cfg=have_zig"),
            "{target}: {stdout}"
        );
        assert!(
            !stdout.contains("cargo:rustc-link-lib"),
            "{target}: {stdout}"
        );
        assert_eq!(fs::read_dir(&out).unwrap().count(), 0);
    }
}
