"""Run the CDX caption IO example in a fresh, offline local Chromium page.

From the repository root, run python3 xtask/oracles/cdx_caption_browser.py.
Requires Python 3.11+, Chrome/Chromium, the wasm32-unknown-unknown Rust target,
cached workspace dependencies and wasm-bindgen-cli matching Cargo.lock. Set
BONDLINE_CHROME and BONDLINE_WASM_BINDGEN to select specific executables.
"""
import base64
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
NAME = "cdx_caption_browser"
TARGET = "wasm32-unknown-unknown"


def chromium():
    candidates = [os.getenv("BONDLINE_CHROME")] if os.getenv("BONDLINE_CHROME") else [
        "chromium", "chromium-browser", "google-chrome", "chrome",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        *(str(Path(os.environ[key]) / "Google/Chrome/Application/chrome.exe")
          for key in ("ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA") if key in os.environ),
    ]
    for name in candidates:
        if path := shutil.which(name):
            return path
    raise SystemExit("Set BONDLINE_CHROME to an installed Chrome or Chromium executable")


def build():
    result = subprocess.run(
        ["cargo", "build", "--locked", "--offline", "--release", "-p", "bondline-io",
         "--example", NAME, "--target", TARGET, "--message-format=json-render-diagnostics"],
        cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE)
    for line in result.stdout.splitlines():
        artifact = json.loads(line)
        if (artifact.get("reason") == "compiler-artifact" and artifact["target"]["name"] == NAME
                and artifact["target"]["kind"] == ["example"]):
            return artifact["executable"]
    raise SystemExit("Cargo did not report the example executable")


def main():
    browser = chromium()
    tool = os.getenv("BONDLINE_WASM_BINDGEN", "wasm-bindgen")
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    version, = (p["version"] for p in packages if p["name"] == "wasm-bindgen")
    if not shutil.which(tool) or subprocess.check_output([tool, "--version"], text=True).strip() != f"wasm-bindgen {version}":
        raise SystemExit(f"Use wasm-bindgen-cli {version} for the current Cargo.lock")
    artifact = build()
    with tempfile.TemporaryDirectory(prefix="bondline-cdx-browser-") as tmp:
        work = Path(tmp)
        subprocess.run([tool, "--target", "no-modules", "--no-typescript", "--out-dir",
                        str(work), artifact], check=True)
        wasm = (work / f"{NAME}_bg.wasm").read_bytes()
        glue = (work / f"{NAME}.js").read_text()
        payload = base64.b64encode(wasm).decode("ascii")
        html = work / "index.html"
        html.write_text("""<!doctype html><meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; connect-src 'none'; script-src 'unsafe-inline' 'wasm-unsafe-eval'">
<output id="status">pending</output><script>""" + glue + f"""
try {{
    const bytes = Uint8Array.from(atob('{payload}'), c => c.charCodeAt(0));
    const instance = wasm_bindgen.initSync({{module:bytes}});
    document.getElementById('status').textContent = 'main=' + instance.main(0, 0);
}} catch (error) {{ document.getElementById('status').textContent = String(error); }}
</script>""", encoding="utf-8")
        result = subprocess.run(
            [browser, "--headless=new", "--dump-dom", "--no-first-run", "--no-default-browser-check",
             "--disable-background-networking", "--disable-extensions", "--disable-sync",
             "--disable-component-update", "--host-resolver-rules=MAP * ~NOTFOUND",
             "--user-data-dir=" + str(work / "profile"), html.as_uri()],
            check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=45)
        status = re.search(rb'<output id="status">([^<]*)</output>', result.stdout)
        if not status or status[1] != b"main=0":
            reason = status[1].decode("utf-8", errors="replace") if status else "no DOM result"
            raise SystemExit(f"Browser CDX IO regression failed ({reason})")
        print(f"CDX IO main=0 | wasm-bindgen {version} | {len(wasm)} bytes | sha256 {sha256(wasm).hexdigest()}")


if __name__ == "__main__":
    main()
