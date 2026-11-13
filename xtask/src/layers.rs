use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    io::Write,
    process::Command,
};

fn expected_layer(name: &str) -> Option<u64> {
    match name {
        "bondline-model" | "bondline-cdxml" | "bondline-cdx" => Some(0),
        "bondline-chem" | "bondline-render" => Some(1),
        "bondline-io" => Some(2),
        "bondline-commands" => Some(3),
        "bondline-mcp" => Some(4),
        "bondline-cli" | "bondline-ui" => Some(5),
        _ => None,
    }
}

fn violations(metadata: &Value) -> Result<Vec<String>, Box<dyn Error>> {
    let members: BTreeSet<_> = metadata["workspace_members"]
        .as_array()
        .ok_or("missing workspace members")?
        .iter()
        .map(|v| v.as_str().ok_or("invalid member id"))
        .collect::<Result<_, _>>()?;
    let packages: BTreeMap<_, _> = metadata["packages"]
        .as_array()
        .ok_or("missing packages")?
        .iter()
        .map(|p| {
            p["id"]
                .as_str()
                .map(|id| (id, p))
                .ok_or("invalid package id")
        })
        .collect::<Result<_, _>>()?;
    let edges: BTreeMap<_, _> = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("missing resolution graph")?
        .iter()
        .map(|node| {
            let id = node["id"].as_str().ok_or("missing graph node id")?;
            let deps: Vec<_> = node["deps"]
                .as_array()
                .ok_or("missing dependencies")?
                .iter()
                .map(|dep| dep["pkg"].as_str().ok_or("invalid dependency id"))
                .collect::<Result<_, _>>()?;
            Ok((id, deps))
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    let mut errors = Vec::new();
    for id in &members {
        let package = packages.get(id).ok_or("missing workspace package")?;
        let name = package["name"].as_str().ok_or("missing package name")?;
        if !name.starts_with("bondline-") {
            continue;
        }
        let Some(layer) = expected_layer(name) else {
            errors.push(format!("unclassified Bondline crate {name}"));
            continue;
        };
        if package["metadata"]["bondline"]["layer"].as_u64() != Some(layer) {
            errors.push(format!("{name} must declare layer {layer}"));
        }
        for dependency in edges.get(id).ok_or("missing workspace graph node")? {
            if !members.contains(dependency) {
                continue;
            }
            let other = packages
                .get(dependency)
                .ok_or("missing dependency package")?;
            let other_name = other["name"].as_str().ok_or("missing dependency name")?;
            if matches!(name, "bondline-cdxml" | "bondline-cdx") {
                errors.push(format!(
                    "standalone format {name} depends on workspace crate {other_name}"
                ));
            }
            if let Some(other_layer) = expected_layer(other_name)
                && other_layer >= layer
            {
                errors.push(format!(
                    "{name} (L{layer}) depends on {other_name} (L{other_layer})"
                ));
            }
        }
        if name == "bondline-ui" {
            continue;
        }
        let mut seen = BTreeSet::new();
        let mut pending = vec![*id];
        while let Some(current) = pending.pop() {
            if !seen.insert(current) {
                continue;
            }
            let current_package = packages.get(current).ok_or("missing transitive package")?;
            let current_name = current_package["name"]
                .as_str()
                .ok_or("missing transitive name")?;
            if matches!(
                current_name,
                "egui" | "eframe" | "winit" | "rfd" | "egui-wgpu" | "egui-winit" | "epaint"
            ) {
                errors.push(format!("{name} reaches GUI dependency {current_name}"));
            }
            pending.extend(
                edges
                    .get(current)
                    .ok_or("missing transitive graph node")?
                    .iter()
                    .copied(),
            );
        }
    }
    Ok(errors)
}

pub fn check() -> Result<(), Box<dyn Error>> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--all-features",
            "--format-version",
            "1",
        ])
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    let metadata = serde_json::from_slice(&output.stdout)?;
    let errors = violations(&metadata)?;
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    writeln!(
        std::io::stdout(),
        "layer graph verified: strict lower-layer edges, standalone parsers, GUI isolation"
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    fn graph(from: &str, from_layer: u64, to: &str, to_layer: u64) -> Value {
        json!({
            "workspace_members": [from, to],
            "packages": [
                {"id":from, "name":from, "metadata":{"bondline":{"layer":from_layer}}},
                {"id":to, "name":to, "metadata":{"bondline":{"layer":to_layer}}}
            ],
            "resolve":{"nodes":[
                {"id":from, "deps":[{"pkg":to}]},
                {"id":to, "deps":[]}
            ]}
        })
    }

    #[test]
    fn dependencies_only_point_to_lower_layers() -> Result<(), Box<dyn std::error::Error>> {
        assert!(super::violations(&graph("bondline-ui", 5, "bondline-render", 1))?.is_empty());
        assert!(!super::violations(&graph("bondline-render", 1, "bondline-ui", 5))?.is_empty());
        assert!(!super::violations(&graph("bondline-cdx", 0, "bondline-model", 0))?.is_empty());
        Ok(())
    }

    #[test]
    fn formats_are_standalone_and_new_crates_need_classification()
    -> Result<(), Box<dyn std::error::Error>> {
        assert!(!super::violations(&graph("bondline-cdxml", 0, "bondline-model", 0))?.is_empty());
        assert!(!super::violations(&graph("bondline-mystery", 5, "bondline-model", 0))?.is_empty());
        assert!(!super::violations(&graph("bondline-model", 5, "bondline-ui", 5))?.is_empty());
        Ok(())
    }

    #[test]
    fn transitive_gui_dependencies_are_rejected_below_the_ui()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = graph("bondline-render", 1, "helper", 0);
        metadata["workspace_members"] = json!(["bondline-render"]);
        metadata["packages"]
            .as_array_mut()
            .ok_or("missing packages")?
            .push(json!({"id":"egui", "name":"egui", "metadata":{}}));
        metadata["resolve"]["nodes"][1]["deps"] = json!([{"pkg":"egui"}]);
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .ok_or("missing nodes")?
            .push(json!({"id":"egui", "deps":[]}));
        let errors = super::violations(&metadata)?;
        assert!(
            errors
                .iter()
                .any(|error| error.contains("GUI dependency egui"))
        );
        Ok(())
    }
}
