#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Build the actual shared Rust browser editor into a standalone local asset directory."""
import argparse
from hashlib import sha256
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]


def runtime_notices():
    """Keep the selected runtime's actual notices, refusing unreviewed missing files."""
    metadata = json.loads(subprocess.check_output([
        'cargo', 'metadata', '--locked', '--offline', '--format-version', '1',
        '--filter-platform', 'wasm32-unknown-unknown'], cwd=ROOT))
    tree = subprocess.check_output([
        'cargo', 'tree', '--locked', '--offline', '--target', 'wasm32-unknown-unknown',
        '--edges', 'normal,no-proc-macro', '--prefix', 'none', '--format', '{p}',
        '-p', 'bondline-ui'], cwd=ROOT, text=True, encoding='utf-8')
    selected = {tuple(line.split()[:2]) for line in tree.splitlines() if line}
    supplement = json.loads((ROOT/'docs/licenses/browser-runtime/manifest.json').read_text(encoding='utf-8'))['files']
    output = ["Bondline browser runtime notices\n\n", (ROOT/'ATTRIBUTION.md').read_text(encoding='utf-8'),
              "\n\nBundled Roboto font\n", (ROOT/'assets/fonts/LICENSE-Roboto.txt').read_text(encoding='utf-8')]
    for name, version in sorted(selected):
        package, = (p for p in metadata['packages'] if p['name'] == name and 'v'+p['version'] == version)
        directory = Path(package['manifest_path']).parent
        files = sorted(p for p in directory.iterdir() if p.is_file() and p.name.lower().startswith(
            ('license', 'licence', 'notice', 'copying', 'copyright')))
        if package['id'] in metadata['workspace_members']:
            files = [ROOT/'LICENSE-MIT', ROOT/'LICENSE-APACHE']
        if not files and name in {'ecolor', 'emath', 'epaint', 'egui-wgpu'} and version == 'v0.29.1':
            vcs = json.loads((directory/'.cargo_vcs_info.json').read_text(encoding='utf-8'))
            if vcs['git']['sha1'] != 'fe368bacc4a15697e347607e73a56c0299b3d42d':
                raise SystemExit(f'Review changed upstream notice provenance: {name}{version}')
            files = [ROOT/'vendor/egui/LICENSE-MIT', ROOT/'vendor/egui/LICENSE-APACHE']
        if not files:
            for record in supplement:
                if record['package'] != name or 'v'+record['version'] != version:
                    continue
                vcs = json.loads((directory/'.cargo_vcs_info.json').read_text(encoding='utf-8'))
                path = ROOT/record['path']
                if vcs['git']['sha1'] != record['revision'] or sha256(path.read_bytes()).hexdigest() != record['sha256']:
                    raise SystemExit(f'Review changed upstream notice provenance: {name}{version}')
                files.append(path)
        if not files:
            raise SystemExit(f'Missing runtime notices; review the pinned public source: {name}{version}')
        output.append(f'\n\n=== {name} {version}: {package["license"]} ===\n')
        for file in files:
            output.extend([f'\n--- {file.name} ---\n', file.read_text(encoding='utf-8')])
        patch = directory/'BONDLINE-PATCH.md'
        if patch.is_file():
            output.extend(['\n--- Reviewed source patch ---\n', patch.read_text(encoding='utf-8')])
    return ''.join(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', type=Path, default=ROOT/'target/bondline-web')
    parser.add_argument('--profile', choices=('dev', 'release'), default='release')
    args = parser.parse_args()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    packages = tomllib.loads((ROOT/'Cargo.lock').read_text(encoding='utf-8'))['package']
    version, = (p['version'] for p in packages if p['name'] == 'wasm-bindgen')
    tool = os.environ.get('BONDLINE_WASM_BINDGEN', 'wasm-bindgen')
    if not shutil.which(tool) or subprocess.check_output([tool, '--version'], text=True, encoding='utf-8').strip() != f'wasm-bindgen {version}':
        raise SystemExit(f'Use wasm-bindgen-cli {version} (set BONDLINE_WASM_BINDGEN)')
    subprocess.run(['cargo', 'run', '--locked', '--offline', '--quiet', '-p', 'xtask', '--', 'licenses'],
                   cwd=ROOT, check=True, stdout=sys.stderr, env={**os.environ, 'CARGO_NET_OFFLINE': 'true'})
    notices = runtime_notices()
    command = ['cargo', 'build', '--locked', '--offline', '-p', 'bondline-ui', '--lib',
               '--target', 'wasm32-unknown-unknown', '--profile', args.profile,
               '--message-format=json-render-diagnostics']
    result = subprocess.run(command, cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True, encoding='utf-8')
    artifact = None
    for line in result.stdout.splitlines():
        record = json.loads(line)
        if record.get('reason') == 'compiler-artifact' and record['target']['name'] == 'bondline_ui':
            artifact = next((x for x in record['filenames'] if x.endswith('.wasm')), artifact)
    if artifact is None:
        raise SystemExit('Cargo did not produce the browser workspace library')
    subprocess.run([tool, '--target', 'web', '--no-typescript', '--out-dir', str(output), artifact], check=True)
    for name in ('index.html', 'bootstrap.js'):
        shutil.copyfile(ROOT/'web'/name, output/name)
    (output/'THIRD-PARTY-NOTICES.txt').write_text(notices, encoding='utf-8', newline='\n')
    template = (ROOT/'web/sw.js').read_text(encoding='utf-8')
    names = ['index.html', 'bootstrap.js', 'bondline_ui.js', 'bondline_ui_bg.wasm', 'THIRD-PARTY-NOTICES.txt']
    hasher = sha256()
    for name in sorted(names):
        hasher.update(name.encode() + b'\0' + (output/name).read_bytes())
    hasher.update(template.encode())
    bundle = hasher.hexdigest()
    (output/'sw.js').write_text(template.replace('__BUNDLE_HASH__', bundle), encoding='utf-8', newline='\n')
    files = {name: {'bytes':(output/name).stat().st_size, 'sha256':sha256((output/name).read_bytes()).hexdigest()}
             for name in [*names, 'sw.js']}
    manifest = {'app':'bondline', 'profile':args.profile, 'wasm_bindgen':version,
                'bundle_sha256':bundle, 'files':files}
    (output/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8', newline='\n')
    print(json.dumps({'directory':str(output), **manifest}, sort_keys=True))


if __name__ == '__main__':
    main()
