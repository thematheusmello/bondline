use serde_json::{Value, json};
use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub struct SdfTestDirectory(PathBuf);

impl SdfTestDirectory {
    pub fn new() -> std::io::Result<Self> {
        let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        fs::create_dir_all(&parent)?;
        loop {
            let path = parent.join(format!(
                "sdf-wire-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for SdfTestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn expected_summary(mixed: bool) -> Value {
    if mixed {
        json!({"format":"sdf","byte_length":777,"record_count":2,"records":[
            {"id":1,"mol_version":"v3000","byte_length":462,"mol_byte_length":344,"field_count":6},
            {"id":2,"mol_version":"v2000","byte_length":315,"mol_byte_length":282,"field_count":1}
        ]})
    } else {
        json!({"format":"sdf","byte_length":690,"record_count":2,"records":[
            {"id":1,"mol_version":"v2000","byte_length":424,"mol_byte_length":282,"field_count":7},
            {"id":2,"mol_version":"v2000","byte_length":266,"mol_byte_length":261,"field_count":0}
        ]})
    }
}

pub fn describe_source(script: &str, source: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!(
            "{}/../../crates/io/tests/{script}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("describe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(source)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "external actual-SDF-file consumer: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

pub fn independently_read_sdf(
    path: &Path,
    expected: &[u8],
    mixed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    assert_eq!(
        bytes, expected,
        "save must preserve the whole source byte-for-byte"
    );
    if mixed {
        let first = describe_source(
            "v3000_oracle.py",
            bytes.get(..344).ok_or("literal V3000 span")?,
        )?;
        let second = describe_source(
            "mol_oracle.py",
            bytes.get(462..744).ok_or("literal V2000 span")?,
        )?;
        assert_eq!(first["source_bytes"], 344);
        assert_eq!(first["atoms"].as_array().ok_or("V3000 atoms")?.len(), 2);
        assert_eq!(first["bonds"].as_array().ok_or("V3000 bonds")?.len(), 1);
        assert_eq!(second["smiles"], "CC");
        assert_eq!(second["formula"], "C2H6");
    } else {
        let raw = describe_source("sdf_oracle.py", &bytes)?;
        assert_eq!(
            raw["source_sha256"],
            "f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a"
        );
        assert_eq!(raw["records"][0]["range"], json!([0, 424]));
        assert_eq!(raw["records"][1]["range"], json!([424, 690]));
        assert_eq!(raw["records"][0]["fields"][3]["dt_number"], "0009");
        assert_eq!(
            raw["records"][0]["fields"][4]["value_lines"],
            json!(["one"])
        );
        assert_eq!(
            raw["records"][0]["fields"][5]["value_lines"],
            json!(["two"])
        );
        assert_eq!(raw["records"][0]["rdkit"]["smiles"], "CC");
        assert_eq!(raw["records"][1]["rdkit"]["smiles"], "CC");
        assert_eq!(
            raw["rdkit_records"]
                .as_array()
                .ok_or("supplier diagnostics")?
                .len(),
            1
        );
    }
    Ok(())
}
