//! Actual CLI and authenticated loopback consumers of the shared caption preview.
use super::{TestResult, capture, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

const SOURCE: &[u8] = include_bytes!("../../../io/tests/fixtures/cdx-caption/caption_rgb16.cdx");
const VECTORS: &str = include_str!("../../../io/tests/fixtures/cdx-caption/caption-vectors.json");

fn request(command: &str, params: Value) -> Value {
    json!({"command":command,"params":params})
}

fn connect(server: &LoopbackServer) -> Result<BufReader<TcpStream>, Box<dyn std::error::Error>> {
    let timeout = Duration::from_secs(5);
    let stream = TcpStream::connect_timeout(&server.address(), timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    Ok(client)
}

#[test]
fn cdx_caption_cli_and_authenticated_socket_share_readonly_capture_and_history() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("caption-wire-原-λ.cdx");
    let output = directory.path("caption-wire-保存-λ.cdx");
    let sentinel = directory.path("caption-wire-guard.cdx");
    fs::write(&input, SOURCE)?;
    fs::write(&sentinel, b"unchanged destination")?;
    let manifest: Value = serde_json::from_str(VECTORS)?;
    let model = &manifest["cases"]
        .as_array()
        .ok_or("missing original cases")?
        .iter()
        .find(|case| case["name"] == "caption_rgb16")
        .ok_or("missing caption original")?["model"];
    let open = request("document.open_cdx", json!({"path":input}));
    let capture_request = request("render.capture", capture());
    let headless = cli(&[open.clone(), capture_request.clone()])?;
    assert_eq!(&headless["document"], model);
    assert_eq!(headless["capture"]["backend"], "cpu");
    let png = headless["capture"]["png_base64"]
        .as_str()
        .filter(|png| !png.is_empty())
        .ok_or("missing caption image")?;

    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = connect(&server)?;
    let opened = socket(&mut client, open)?;
    assert_eq!(&opened["result"]["document"], model);
    assert_eq!(opened["result"]["revision"], 1);
    let inspect = request("document.inspect", json!({}));
    let before = socket(&mut client, inspect.clone())?;
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    let image = socket(&mut client, capture_request.clone())?;
    assert_eq!(image["result"]["capture"]["png_base64"].as_str(), Some(png));
    assert_eq!(image["result"]["changed"], false);
    let refused = socket(
        &mut client,
        request(
            "document.save_cdxml",
            json!({"path":sentinel,"overwrite":true}),
        ),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(fs::read(&sentinel)?, b"unchanged destination");
    assert_eq!(socket(&mut client, inspect)?, before);
    assert_eq!(changes.load(Ordering::SeqCst), 1);

    let resized = socket(
        &mut client,
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":612.0,"height_pt":792.0}),
        ),
    )?;
    assert_eq!(resized["result"]["changed"], true);
    assert_eq!(resized["result"]["source"]["modified"], true);
    let refused = socket(
        &mut client,
        request(
            "text.set_run",
            json!({"expected_revision":2,"page_id":1,"text_id":1,"run_index":0,"text":"Edit"}),
        ),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    let refused = socket(&mut client, capture_request.clone())?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    let refused = socket(
        &mut client,
        request(
            "document.save_cdx",
            json!({"path":sentinel,"overwrite":true}),
        ),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(fs::read(&sentinel)?, b"unchanged destination");

    let undone = socket(&mut client, request("history.undo", json!({})))?;
    assert_eq!(&undone["result"]["document"], model);
    assert_eq!(undone["result"]["source"]["modified"], false);
    let restored = socket(&mut client, capture_request)?;
    assert_eq!(
        restored["result"]["capture"]["png_base64"].as_str(),
        Some(png)
    );
    let saved = socket(
        &mut client,
        request("document.save_cdx", json!({"path":output})),
    )?;
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    server.shutdown()?;
    Ok(())
}

#[test]
fn ordered_caption_cli_renders_the_same_stack_on_cpu_and_gpu() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let front_first = directory.path("front-first.cdx");
    let back_first = directory.path("back-first.cdx");
    fs::write(
        &front_first,
        include_bytes!("../../../io/tests/fixtures/cdx-caption/ordered/front_first.cdx"),
    )?;
    fs::write(
        &back_first,
        include_bytes!("../../../io/tests/fixtures/cdx-caption/ordered/back_first.cdx"),
    )?;
    let witness: Value = serde_json::from_str(include_str!(
        "../../../io/tests/fixtures/cdx-caption/ordered/cases.json"
    ))?;
    let expected = &witness["front_first"]["model"];
    let frame = |path: &std::path::Path, backend| {
        cli(&[
            request("document.open_cdx", json!({"path":path})),
            request(
                "render.capture",
                json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":backend}),
            ),
        ])
    };
    let cpu = frame(&front_first, "cpu")?;
    let png = cpu["capture"]["png_base64"]
        .as_str()
        .filter(|image| !image.is_empty())
        .ok_or("missing ordered preview")?;
    let reordered = frame(&back_first, "cpu")?;
    let gpu = frame(&front_first, "gpu")?;
    for captured in [&cpu, &reordered, &gpu] {
        assert_eq!(&captured["document"], expected);
        assert_eq!(captured["capture"]["png_base64"].as_str(), Some(png));
        assert_eq!(captured["source"]["modified"], false);
        assert_eq!(captured["undo_depth"], 0);
    }
    Ok(())
}

#[test]
fn explicit_caption_runs_share_the_real_cli_render_and_exact_save_paths() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("styled-原-λ.cdx");
    let output = directory.path("styled-再開-β.cdx");
    let raw = include_bytes!("../../../io/tests/fixtures/cdx-caption/runs/mixed_runs.cdx");
    fs::write(&input, raw)?;
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../io/tests/fixtures/cdx-caption/runs/cases.json"
    ))?;
    let model = &manifest["cases"]["mixed_runs"]["model"];
    let open = request("document.open_cdx", json!({"path":input}));
    let cpu = cli(&[open.clone(), request("render.capture", capture())])?;
    let mut gpu = capture();
    gpu["backend"] = json!("gpu");
    let metal_or_native_gpu = cli(&[open.clone(), request("render.capture", gpu)])?;
    let image = &cpu["capture"]["png_base64"];
    assert!(image.as_str().is_some_and(|png| !png.is_empty()));
    assert_eq!(&cpu["document"], model);
    assert_eq!(&metal_or_native_gpu["document"], model);
    assert_eq!(&metal_or_native_gpu["capture"]["png_base64"], image);
    assert_eq!(metal_or_native_gpu["capture"]["backend"], "gpu");

    let reopened = cli(&[
        open,
        request("document.save_cdx", json!({"path":output})),
        request("document.open_cdx", json!({"path":output})),
        request("render.capture", capture()),
    ])?;
    assert_eq!(fs::read(&output)?, raw);
    assert_eq!(&reopened["document"], model);
    assert_eq!(&reopened["capture"]["png_base64"], image);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["undo_depth"], 0);
    Ok(())
}

#[test]
fn cdx_caption_run_edit_round_trips_through_cli_and_authenticated_loopback() -> TestResult {
    let before = include_bytes!("../../../io/tests/fixtures/cdx-caption-edit/multi_before.cdx");
    let after = include_bytes!("../../../io/tests/fixtures/cdx-caption-edit/multi_after.cdx");
    let witnesses: Value = serde_json::from_str(include_str!(
        "../../../io/tests/fixtures/cdx-caption-edit/cases.json"
    ))?;
    let case = &witnesses["cases"]["multi_shift"];
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("caption-run-原-λ.cdx");
    let cli_output = directory.path("caption-run-cli-保存-λ.cdx");
    let loopback_output = directory.path("caption-run-loopback-保存-β.cdx");
    fs::write(&input, before)?;
    let open = request("document.open_cdx", json!({"path":input}));
    let mut params = case["params"].clone();
    params["expected_revision"] = json!(1);
    let edit = request("text.set_run", params.clone());
    let render = request("render.capture", capture());
    let reopened = cli(&[
        open.clone(),
        edit.clone(),
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdx", json!({"path":cli_output})),
        request("document.open_cdx", json!({"path":cli_output})),
        render.clone(),
    ])?;
    assert_eq!(reopened["document"], case["after_model"]);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(fs::read(&cli_output)?, after);
    let png = reopened["capture"]["png_base64"]
        .as_str()
        .filter(|png| !png.is_empty())
        .ok_or("missing edited caption PNG")?;

    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = connect(&server)?;
    let opened = socket(&mut client, open)?;
    assert_eq!(opened["result"]["document"], case["before_model"]);
    let edited = socket(&mut client, edit)?;
    assert_eq!(edited["result"]["document"], case["after_model"]);
    assert_eq!(edited["result"]["undo_depth"], 1);
    assert_eq!(edited["result"]["file"]["dirty"], true);
    let inspect = request("document.inspect", json!({}));
    let edited_state = socket(&mut client, inspect.clone())?;
    assert_eq!(changes.load(Ordering::SeqCst), 2);
    params["expected_revision"] = json!(2);
    let noop = socket(&mut client, request("text.set_run", params))?;
    assert_eq!(noop["result"]["changed"], false);
    assert_eq!(noop["result"]["revision"], 2);
    assert_eq!(socket(&mut client, inspect)?, edited_state);
    assert_eq!(changes.load(Ordering::SeqCst), 2);

    let saved = socket(
        &mut client,
        request("document.save_cdx", json!({"path":loopback_output})),
    )?;
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(
        saved["result"]["source"],
        json!({"format":"cdx","byte_length":before.len(),"modified":true})
    );
    assert_eq!(fs::read(&input)?, before);
    assert_eq!(fs::read(&loopback_output)?, after);
    let opened = socket(
        &mut client,
        request("document.open_cdx", json!({"path":loopback_output})),
    )?;
    assert_eq!(opened["result"]["document"], case["after_model"]);
    assert_eq!(opened["result"]["undo_depth"], 0);
    let rendered = socket(&mut client, render)?;
    assert_eq!(
        rendered["result"]["capture"]["png_base64"].as_str(),
        Some(png)
    );
    drop(client);
    server.shutdown()?;
    Ok(())
}

#[test]
fn mixed_unicode_run_edits_survive_history_and_editable_reopen() -> TestResult {
    let before =
        include_bytes!("../../../io/tests/fixtures/cdx-mixed-caption-edit/extended_before.cdx");
    let after = include_bytes!("../../../io/tests/fixtures/cdx-mixed-caption-edit/after_both.cdx");
    let prefix =
        include_bytes!("../../../io/tests/fixtures/cdx-mixed-caption-edit/after_prefix.cdx");
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("Original β.cdx");
    let output = dir.path("Modified λ.cdx");
    let again = dir.path("Reopened Ω.cdx");
    fs::write(&input, before)?;
    let edit = |revision, run, body| {
        request(
            "text.set_run",
            json!({"expected_revision":revision,"page_id":1,"text_id":19,"run_index":run,"text":body}),
        )
    };
    let state = cli(&[
        request("document.open_cdx", json!({"path":input})),
        edit(1, 0, "H H "),
        edit(2, 1, "λéπΩ"),
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdx", json!({"path":output})),
        request("document.open_cdx", json!({"path":output})),
    ])?;
    assert_eq!(fs::read(&input)?, before);
    assert_eq!(fs::read(&output)?, after);
    assert_eq!(state["file"]["dirty"], false);
    assert_eq!(state["undo_depth"], 0);
    let text = &state["document"]["pages"][0]["texts"][1];
    assert_eq!(text["id"], 19);
    assert_eq!(text["runs"][1]["text"], "λéπΩ");
    cli(&[
        request("document.open_cdx", json!({"path":output})),
        edit(1, 1, "λΩé"),
        request("document.save_cdx", json!({"path":again})),
    ])?;
    assert_eq!(fs::read(&again)?, prefix);
    Ok(())
}
