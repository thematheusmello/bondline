use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    error::Error,
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
};

const TARGETS: &[&str] = &[
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

fn allowed(license: &str) -> bool {
    let legacy = match license {
        "MIT/Apache-2.0" | "Apache-2.0/MIT" => "MIT OR Apache-2.0",
        other => other,
    };
    spdx::Expression::parse(legacy).is_ok_and(|expression| {
        expression.evaluate(|requirement| {
            requirement.addition.is_none()
                && requirement.license.id().is_some_and(|id| {
                    matches!(
                        id.name,
                        "MIT" | "Apache-2.0" | "BSD-2-Clause" | "BSD-3-Clause"
                    )
                })
        })
    })
}

fn runtime_packages(tree: &str) -> Result<BTreeSet<(String, String)>, Box<dyn Error>> {
    tree.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next().ok_or("missing Cargo package name")?;
            let version = fields
                .next()
                .and_then(|s| s.strip_prefix('v'))
                .ok_or("invalid Cargo package version")?;
            Ok((name.to_owned(), version.to_owned()))
        })
        .collect()
}

fn verify_asset(bytes: &[u8], license: &str, expected_hash: &str) -> Result<(), Box<dyn Error>> {
    if !allowed(license) {
        return Err(format!("asset license outside policy: {license}").into());
    }
    if format!("{:x}", Sha256::digest(bytes)) != expected_hash {
        return Err("asset source hash mismatch".into());
    }
    Ok(())
}

fn confined_path(root: &Path, relative: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("asset path must be relative without traversal".into());
    }
    let path = root.join(path).canonicalize()?;
    if !path.starts_with(root.canonicalize()?) {
        return Err("asset symlink escapes repository".into());
    }
    Ok(path)
}

fn check_assets() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing workspace root")?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("assets/manifest.json"))?)?;
    let files = manifest["files"]
        .as_array()
        .ok_or("missing asset inventory")?;
    for file in files {
        let path = confined_path(root, file["path"].as_str().ok_or("missing asset path")?)?;
        let notice = confined_path(root, file["notice"].as_str().ok_or("missing asset notice")?)?;
        if !notice.is_file() || notice.metadata()?.len() == 0 {
            return Err("missing nonempty asset license notice".into());
        }
        verify_asset(
            &std::fs::read(path)?,
            file["license"].as_str().ok_or("missing asset license")?,
            file["sha256"].as_str().ok_or("missing asset hash")?,
        )?;
        if file["source"].as_str().is_none() || file["revision"].as_str().is_none() {
            return Err("missing pinned asset provenance".into());
        }
    }
    writeln!(
        std::io::stdout(),
        "{} bundled assets: licenses, hashes and notice files checked",
        files.len()
    )?;
    Ok(())
}

pub fn check() -> Result<(), Box<dyn Error>> {
    check_assets()?;
    for target in TARGETS {
        let output = Command::new("cargo")
            .args([
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--filter-platform",
                target,
            ])
            .output()?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
        }
        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let output = Command::new("cargo")
            .args([
                "tree",
                "--locked",
                "--target",
                target,
                "--edges",
                "normal,no-proc-macro",
                "--package",
                "bondline-ui",
                "--prefix",
                "none",
                "--format",
                "{p}",
            ])
            .output()?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
        }
        // Cargo resolves the real target/feature graph; metadata alone overapproximates it.
        let shipped = runtime_packages(std::str::from_utf8(&output.stdout)?)?;
        let mut denied = Vec::new();
        for (name, version) in &shipped {
            let matches: Vec<_> = metadata["packages"]
                .as_array()
                .ok_or("missing Cargo packages")?
                .iter()
                .filter(|p| p["name"] == *name && p["version"] == *version)
                .collect();
            let [package] = matches.as_slice() else {
                return Err("missing or ambiguous runtime package provenance".into());
            };
            let license = package["license"].as_str().unwrap_or("");
            if !allowed(license) {
                denied.push(format!("{name} {version}: {license}"));
            }
        }
        if !denied.is_empty() {
            return Err(format!(
                "{target}: runtime dependencies outside strict policy:\n{}",
                denied.join("\n")
            )
            .into());
        }
        writeln!(
            std::io::stdout(),
            "{target}: {} runtime packages; declared licenses allowed (build tools excluded)",
            shipped.len()
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{allowed, confined_path, runtime_packages, verify_asset};

    #[test]
    fn asset_paths_stay_inside_the_workspace() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("missing root")?;
        assert!(confined_path(root, "README.md").is_ok());
        assert!(confined_path(root, "../README.md").is_err());
        assert!(confined_path(root, "/tmp").is_err());
        Ok(())
    }

    #[test]
    fn assets_require_an_allowed_license_and_exact_source_hash()
    -> Result<(), Box<dyn std::error::Error>> {
        let empty_sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        verify_asset(b"", "Apache-2.0", empty_sha)?;
        assert!(verify_asset(b"changed", "Apache-2.0", empty_sha).is_err());
        assert!(verify_asset(b"", "OFL-1.1", empty_sha).is_err());
        Ok(())
    }

    #[test]
    fn runtime_tree_uses_cargo_target_selection_and_deduplicates()
    -> Result<(), Box<dyn std::error::Error>> {
        let packages = runtime_packages(
            "bondline-ui v0.1.0 (/project with spaces)\nserde v1.0.229\nserde v1.0.229 (*)\n",
        )?;
        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&("serde".to_owned(), "1.0.229".to_owned())));
        assert!(runtime_packages("invalid cargo output").is_err());
        Ok(())
    }

    #[test]
    fn permits_an_allowed_alternative_but_requires_every_conjunction() {
        assert!(allowed("MIT OR Zlib"));
        assert!(allowed("MIT AND BSD-3-Clause"));
        assert!(!allowed("MIT AND Unicode-3.0"));
        assert!(!allowed("ISC"));
        assert!(!allowed("CC0-1.0"));
        assert!(!allowed("Zlib"));
        assert!(!allowed(""));
        assert!(!allowed("MIT OR nonsense"));
    }

    #[test]
    fn handles_legacy_dual_license_spelling_without_broadening_policy() {
        assert!(allowed("MIT/Apache-2.0"));
        assert!(!allowed("ISC/Zlib"));
    }
}
