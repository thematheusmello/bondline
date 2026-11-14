use crate::{
    Cancellation,
    label::{self, Layout},
};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn oracle(
    label: &label::Label,
    symbol: &str,
    side: &str,
    hydrogens: u16,
) -> TestResult<std::process::Output> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(interpreter)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/display_label_oracle.py"
        ))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/Roboto-Regular.ttf"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let boxes = label
        .pixels
        .iter()
        .map(|r| [r.x, r.y, r.width, r.height])
        .collect::<Vec<_>>();
    let mut input = child.stdin.take().ok_or("stdin")?;
    serde_json::to_writer(
        &mut input,
        &serde_json::json!({"symbol":symbol,"side":side,"hydrogens":hydrogens,"boxes":boxes}),
    )?;
    input.flush()?;
    drop(input);
    Ok(child.wait_with_output()?)
}

#[test]
fn stacked_literal_h_matches_independent_freetype_without_changing_old_oracle_policy() -> TestResult
{
    let mut minimum = 1.0_f64;
    for (number, symbol) in [(6, "C"), (7, "N"), (8, "O")] {
        for (layout, side) in [(Layout::Above, "above"), (Layout::Below, "below")] {
            for h in [1, 2, 12, u16::MAX] {
                let label = label::resolved_element(
                    [400.0, 200.0],
                    bondline_model::Element::try_from(number)?,
                    layout,
                    h,
                    8.0,
                    [800, 400],
                    &Cancellation::default(),
                )?;
                let output = oracle(&label, symbol, side, h)?;
                assert!(
                    output.status.success(),
                    "{symbol}/{side}/H{h}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
                minimum = minimum.min(value["freetype_ink_iou"].as_f64().ok_or("score")?);
                assert_eq!(label.masks().count(), 2);
            }
        }
    }
    let label = label::resolved_element(
        [400.0, 200.0],
        bondline_model::Element::try_from(7)?,
        Layout::Above,
        2,
        8.0,
        [800, 400],
        &Cancellation::default(),
    )?;
    for (symbol, side, h) in [("N", "below", 2), ("O", "above", 2), ("N", "above", 0)] {
        assert!(
            !oracle(&label, symbol, side, h)?.status.success(),
            "counterfactual {symbol}/{side}/H{h} accepted"
        );
    }
    println!(
        "24 stacked whole-label independent FreeType cases: minimumIoU={minimum}; three semantic controls rejected"
    );
    Ok(())
}
