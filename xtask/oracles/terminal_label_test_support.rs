//! Original fixed test source and expectations; no product model/writer builds this input.
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
type TestResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type Flags = [(Option<bool>, Option<bool>); 3];
pub const ALL_FALSE: Flags = [(Some(false), Some(false)); 3];
pub const HIDDEN:&[u8]=br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="420" Height="280"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="144 144" NumHydrogens="3" ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="no"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="208 144" NumHydrogens="2" ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="no"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="240 96" NumHydrogens="3" ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="no"/><b id="6" B="3" E="4" Order="1"/><b id="7" B="4" E="5" Order="1"/></fragment></page></CDXML>"#;

pub fn fragment(flags: &Flags) -> Value {
    let positions = [[144.0, 144.0], [208.0, 144.0], [240.0, 96.0]];
    let atoms = positions
        .into_iter()
        .zip([3, 2, 3])
        .zip(flags)
        .enumerate()
        .map(|(i, ((p, h), (t, n)))| {
            let mut atom = json!({"id":i+1,"element":6,"position":{"x_pt":p[0],"y_pt":p[1]},
            "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h});
            if let Some(t) = t {
                atom["show_terminal_carbon_labels"] = json!(t);
            }
            if let Some(n) = n {
                atom["show_non_terminal_carbon_labels"] = json!(n);
            }
            atom
        })
        .collect::<Vec<_>>();
    json!({"id":1,"atoms":atoms,"bonds":[{"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"single"}]})
}
pub fn source(flags: &Flags) -> TestResult<Vec<u8>> {
    let mut source = String::from_utf8(HIDDEN.to_vec())?;
    for (i, (t, n)) in flags.iter().enumerate() {
        let marker = format!("<n id=\"{}\" ", i + 3);
        let start = source.find(&marker).ok_or("fixed node")?;
        let end = start + source[start..].find("/>").ok_or("fixed node end")? + 2;
        let mut node = source[start..end].to_owned();
        for (attribute, value) in [
            ("ShowTerminalCarbonLabels", t),
            ("ShowNonTerminalCarbonLabels", n),
        ] {
            let before = format!(" {attribute}=\"no\"");
            let after = value.map_or_else(String::new, |show| {
                format!(" {attribute}=\"{}\"", if show { "yes" } else { "no" })
            });
            assert_eq!(node.matches(&before).count(), 1);
            node = node.replace(&before, &after);
        }
        source.replace_range(start..end, &node);
    }
    Ok(source.into_bytes())
}
pub fn consume(path: &Path, flags: &Flags) -> TestResult<()> {
    let raw = fs::read(path)?;
    // Complete literal bytes independently check terminal+nonterminal values AND order.
    assert_eq!(raw, source(flags)?);
    let mut expected = fragment(flags);
    for atom in expected["atoms"].as_array_mut().ok_or("atoms")? {
        let atom = atom.as_object_mut().ok_or("atom")?;
        atom.remove("show_terminal_carbon_labels");
        atom.remove("show_non_terminal_carbon_labels");
    }
    let n = flags.iter().map(|(_, n)| *n).collect::<Vec<_>>();
    let args = serde_json::to_string(&json!({"graph":[expected],"smiles":"CCC","formula":"C3H8",
        "flags":n,"double_positions":[null,null]}))?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../crates/commands/tests/ring_create/named_ring_oracle.py"
            ),
            &args,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("chemical consumer stdin")?
        .write_all(&raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
