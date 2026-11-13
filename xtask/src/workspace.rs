use std::{
    error::Error,
    path::{Path, PathBuf},
    process::Command,
};

pub fn root() -> Result<PathBuf, Box<dyn Error>> {
    root_at(&std::env::current_dir()?)
}

fn root_at(directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let output = Command::new("cargo")
        .args([
            "locate-project",
            "--workspace",
            "--message-format",
            "json",
            "--frozen",
        ])
        .current_dir(directory)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "cannot locate invoked workspace: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let location: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let manifest = Path::new(
        location
            .get("root")
            .and_then(serde_json::Value::as_str)
            .ok_or("Cargo did not return a workspace manifest")?,
    );
    if !manifest.is_absolute() {
        return Err("Cargo returned a relative workspace manifest".into());
    }
    manifest
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "missing workspace root".into())
}

#[cfg(test)]
mod tests {
    use super::root_at;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Directory(PathBuf);

    impl Directory {
        fn new() -> std::io::Result<Self> {
            let path = std::env::temp_dir().join(format!(
                "bondline-workspace-λ-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }

    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn locator_uses_the_invoked_workspace_from_root_and_nested_member()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        fs::write(
            directory.0.join("Cargo.toml"),
            "[workspace]\nmembers=[\"member\"]\nresolver=\"3\"\n",
        )?;
        let member = directory.0.join("member");
        let nested = member.join("src");
        fs::create_dir_all(&nested)?;
        fs::write(
            member.join("Cargo.toml"),
            "[package]\nname=\"bondline-locator-probe\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
        )?;
        fs::write(nested.join("lib.rs"), "")?;
        for invoked in [&directory.0, &member, &nested] {
            assert_eq!(
                root_at(invoked)?.canonicalize()?,
                directory.0.canonicalize()?
            );
        }
        assert!(!directory.0.join("Cargo.lock").exists());
        Ok(())
    }

    #[test]
    fn failed_locator_never_falls_back_to_the_build_time_checkout()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        fs::write(directory.0.join("Cargo.toml"), "[workspace\n")?;
        assert!(root_at(&directory.0).is_err());
        Ok(())
    }
}
