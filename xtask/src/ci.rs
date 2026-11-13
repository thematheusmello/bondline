use std::{error::Error, io::Write, path::Path, process::Command};

pub fn check() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing workspace root")?;
    std::env::set_current_dir(root)?;
    for arguments in [
        ["fmt", "--all", "--", "--check"].as_slice(),
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        &["test", "--workspace", "--locked"],
        #[cfg(target_os = "macos")]
        &["test", "-p", "accesskit_macos", "--lib", "--locked"],
        #[cfg(target_os = "macos")]
        &[
            "clippy",
            "-p",
            "accesskit_macos",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    ] {
        writeln!(std::io::stdout(), "checking: cargo {}", arguments.join(" "))?;
        if !Command::new("cargo").args(arguments).status()?.success() {
            return Err(format!("CI failed: cargo {}", arguments.join(" ")).into());
        }
    }
    super::layers::check()?;
    super::licenses::check()?;
    super::corpus::check(false)?;
    writeln!(
        std::io::stdout(),
        "local CI checks passed; see verification.md for platform/feature scope"
    )?;
    Ok(())
}
