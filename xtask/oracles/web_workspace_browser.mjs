// SPDX-License-Identifier: MIT OR Apache-2.0
// An owned Chromium profile drives the actual Rust canvas and verifies browser recovery.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile, writeFile, mkdtemp, mkdir, rm} from 'node:fs/promises';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import path from 'node:path';

const assets = path.resolve(process.argv[2] ?? 'target/bondline-web');
const output = path.resolve(process.argv[3] ?? 'target/web-workspace-browser');
await mkdir(output, {recursive:true});
const manifest = JSON.parse(await readFile(path.join(assets, 'manifest.json'), 'utf8'));
for (const [name,record] of Object.entries(manifest.files)) {
    const bytes = await readFile(path.join(assets,name));
    assert.equal(bytes.length,record.bytes);
    assert.equal(createHash('sha256').update(bytes).digest('hex'),record.sha256);
}
const allowed = new Set(Object.keys(manifest.files));
let localNetworkAvailable = true, blockedRequests = 0;
const server = createServer(async (request, response) => {
    if (!localNetworkAvailable) {blockedRequests++; request.socket.destroy(); return;}
    try {
        const url = new URL(request.url, 'http://localhost');
        const name = url.pathname === '/' ? 'index.html' : url.pathname.slice(1);
        if (request.method !== 'GET' || !allowed.has(name)) {response.writeHead(404).end(); return;}
        response.setHeader('Content-Type', name.endsWith('.wasm') ? 'application/wasm' : name.endsWith('.js') ? 'text/javascript' : name.endsWith('.txt') ? 'text/plain' : 'text/html');
        response.end(await readFile(path.join(assets, name)));
    } catch {response.writeHead(404).end();}
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const profile = await mkdtemp(path.join(tmpdir(), 'bondline-web-workspace-'));
const browser = process.env.BONDLINE_CHROME ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const chrome = spawn(browser, ['--headless=new', '--no-first-run', '--no-default-browser-check',
    '--disable-background-networking', '--disable-extensions', '--disable-sync', '--disable-component-update',
    '--enable-unsafe-webgpu', '--window-size=1160,800', '--force-device-scale-factor=1',
    '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank'], {stdio:['ignore','ignore','pipe']});
let stderr = '';
chrome.stderr.on('data', bytes => {stderr = (stderr + bytes).slice(-30000);});
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
const clients = [];
class Page {
    constructor(socket, id) {
        this.socket = socket; this.id = id; this.sequence = 0; this.pending = new Map(); this.events = [];
        socket.addEventListener('message', event => {
            const data = JSON.parse(event.data);
            if (data.method?.startsWith('ServiceWorker.') || ['Runtime.exceptionThrown','Runtime.consoleAPICalled','Log.entryAdded','Page.javascriptDialogOpening'].includes(data.method)) this.events.push(data);
            if (data.id) {
                const pending = this.pending.get(data.id);
                if (pending) {
                    clearTimeout(pending.timer); this.pending.delete(data.id);
                    if (data.error) pending.reject(Error(JSON.stringify(data.error)));
                    else pending.resolve(data.result);
                }
            }
        });
    }
    call(method, params={}) {
        return new Promise((resolve, reject) => {
            const id = ++this.sequence;
            const timer = setTimeout(() => {this.pending.delete(id); reject(Error(`CDP timeout: ${method}`));}, 15000);
            this.pending.set(id, {resolve,reject,timer});
            this.socket.send(JSON.stringify({id,method,params}));
        });
    }
    async evaluate(expression) {
        const value = await this.call('Runtime.evaluate', {expression, returnByValue:true, awaitPromise:true});
        if (value.exceptionDetails) throw Error(JSON.stringify(value.exceptionDetails));
        return value.result.value;
    }
    command(command, params={}) {
        return this.evaluate(`JSON.parse(bondline.execute(${JSON.stringify(JSON.stringify({command,params}))}))`);
    }
    async wait(expression, seconds=40) {
        const end = Date.now() + seconds*1000;
        while (Date.now() < end) {
            const result = await this.evaluate(expression);
            if (result) return result;
            await delay(60);
        }
        throw Error(`Browser wait timed out: ${expression}`);
    }
    async ready() {
        const result = await this.wait('window.bondlineReady || window.bondlineStartupError || false');
        assert.equal(result, true, 'the actual Rust/WebGPU workspace must start');
        await this.wait('(() => {try{return !!JSON.parse(bondline.canvas_geometry()).width}catch{return false}})()');
    }
    async screenshot(name) {
        const image = await this.call('Page.captureScreenshot', {format:'png'});
        await writeFile(path.join(output, name), Buffer.from(image.data, 'base64'));
    }
}
let port;
async function connect(target) {
    const socket = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((resolve,reject) => {socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
    const page = new Page(socket, target.id); clients.push(page);
    await page.call('Page.enable'); await page.call('Runtime.enable'); await page.call('ServiceWorker.enable');
    await page.call('Emulation.setDeviceMetricsOverride', {width:1160,height:800,deviceScaleFactor:1,mobile:false});
    return page;
}
async function newPage() {
    const target = await (await fetch(`http://127.0.0.1:${port}/json/new?${origin}`, {method:'PUT'})).json();
    const page = await connect(target); await page.ready(); return page;
}
async function click(page, x, y) {
    await page.call('Input.dispatchMouseEvent', {type:'mouseMoved',x,y}); await delay(80);
    await page.call('Input.dispatchMouseEvent', {type:'mousePressed',x,y,button:'left',buttons:1,clickCount:1}); await delay(80);
    await page.call('Input.dispatchMouseEvent', {type:'mouseReleased',x,y,button:'left',buttons:0,clickCount:1}); await delay(100);
}
async function drag(page, x, y) {
    await page.call('Input.dispatchMouseEvent', {type:'mouseMoved',x:x-60,y}); await delay(80);
    await page.call('Input.dispatchMouseEvent', {type:'mousePressed',x:x-60,y,button:'left',buttons:1,clickCount:1}); await delay(80);
    await page.call('Input.dispatchMouseEvent', {type:'mouseMoved',x:x+60,y,buttons:1}); await delay(80);
    await page.call('Input.dispatchMouseEvent', {type:'mouseReleased',x:x+60,y,button:'left',buttons:0,clickCount:1});
}
async function shortcut(page, redo) {
    const mac = await page.evaluate('navigator.platform.startsWith("Mac")');
    const modifiers = (mac ? 4 : 2) | (redo ? 8 : 0);
    await page.call('Input.dispatchKeyEvent', {type:'keyDown',key:redo?'Z':'z',code:'KeyZ',windowsVirtualKeyCode:90,modifiers});
    await page.call('Input.dispatchKeyEvent', {type:'keyUp',key:redo?'Z':'z',code:'KeyZ',windowsVirtualKeyCode:90,modifiers});
}
const report = {origin, bundle:manifest.bundle_sha256, profile:manifest.profile, steps:[]};
try {
    for (let i=0;i<100;i++) {
        try {port = Number((await readFile(path.join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0]); break;}
        catch {await delay(100);}
    }
    assert(port, 'owned Chrome must expose its isolated CDP endpoint');
    const target = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find(target => target.type==='page');
    let page = await connect(target);
    report.browser = await page.call('Browser.getVersion');
    await page.call('Page.navigate', {url:origin}); await page.ready();
    const initial = await page.command('document.inspect');
    assert.deepEqual(initial.document.pages[0].fragments, []);
    assert.equal(initial.ui_file.available, false, 'native filesystem host stays unavailable');
    assert.equal((await page.command('document.open_cdxml',{path:'/browser-is-not-a-filesystem.cdxml'})).error.code,'backend_unavailable');
    assert.equal((await page.command('ui.file',{expected_revision:initial.revision,action:'open'})).error.code,'ui_unavailable');
    assert.equal(await page.evaluate('(() => {try {bondline.execute("x".repeat(1024*1024+1));return false}catch{return true}})()'),true);
    assert.equal(await page.evaluate('(() => {try {bondline.execute({command:"document.inspect"});return false}catch{return true}})()'),true);
    const utf8Bound = await page.evaluate('JSON.parse(bondline.execute("β".repeat(600000)))');
    assert.equal(utf8Bound.error.code,'invalid_params'); assert.match(utf8Bound.error.message,/1 MiB/);
    assert.deepEqual((await page.command('document.inspect')).document,initial.document);
    report.steps.push('native file/presentation and oversized/non-string requests refuse without edits');
    const registry = await page.evaluate('JSON.parse(bondline.registry())');
    for (const name of ['document.inspect', 'bond.create', 'history.undo', 'history.redo']) {
        assert.equal(registry.filter(command => command.name === name).length, 1);
    }
    report.registry_count = registry.length;
    await writeFile(path.join(output, 'registry.json'), JSON.stringify(registry));
    const key = await page.evaluate('bondline.recovery_key()');
    const edit = {page_id:1,begin_x_pt:100,begin_y_pt:120,end_x_pt:148,end_y_pt:120};
    const captionSource = await readFile(new URL('../../crates/io/tests/fixtures/captions/explicit.cdxml',import.meta.url),'utf8');
    const malformedUtf16 = captionSource.replace('<fonttable>',`<!--${String.fromCharCode(0xd800)}--><fonttable>`);
    assert.equal(malformedUtf16.isWellFormed(),false);
    await page.evaluate(`localStorage.setItem(${JSON.stringify(key)},${JSON.stringify(malformedUtf16)})`);
    await page.call('Target.closeTarget',{targetId:page.id});
    const malformed = await newPage();
    const admitted = await malformed.evaluate('JSON.parse(bondline.host_status())');
    const textEdit = await malformed.command('text.set_run',{expected_revision:0,page_id:1,text_id:1,run_index:0,text:'Phase B'});
    const retained = await malformed.evaluate(`localStorage.getItem(${JSON.stringify(key)}) === ${JSON.stringify(malformedUtf16)}`);
    report.malformed_utf16 = {admitted:admitted.recovery_allowed,changed:textEdit.changed??false,retained};
    assert.equal(retained,true,'invalid UTF-16 outside the edited caption must never be normalized and overwritten');
    assert.equal(admitted.recovery_allowed,false);
    assert.equal(await malformed.evaluate('(() => {try {bondline.execute(String.fromCharCode(0xd800));return false}catch(error){return String(error).includes("Unicode")}})()'),true);
    await malformed.command('bond.create',edit);
    assert.equal(await malformed.evaluate(`localStorage.getItem(${JSON.stringify(key)}) === ${JSON.stringify(malformedUtf16)}`),true);
    report.steps.push('malformed UTF-16 storage is rejected before Unicode conversion and kept through later edits');
    const validUnicode = captionSource.replace('<fonttable>','<!--β🧪--><fonttable>');
    await malformed.evaluate(`localStorage.setItem(${JSON.stringify(key)},${JSON.stringify(validUnicode)})`);
    await malformed.call('Target.closeTarget',{targetId:malformed.id});
    const unicode = await newPage();
    assert.equal((await unicode.evaluate('JSON.parse(bondline.host_status())')).recovery_allowed,true);
    assert.equal((await unicode.command('text.set_run',{expected_revision:0,page_id:1,text_id:1,run_index:0,text:'Phase é'})).changed,true);
    assert.equal(await unicode.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),validUnicode.replace('Phase A','Phase é'));
    const counterpart = captionSource.replace('<fonttable>','<!--�--><fonttable>');
    await unicode.evaluate(`localStorage.setItem(${JSON.stringify(key)},${JSON.stringify(counterpart)})`);
    await unicode.call('Target.closeTarget',{targetId:unicode.id});
    const replaced = await newPage();
    assert.equal((await replaced.evaluate('JSON.parse(bondline.host_status())')).recovery_allowed,true);
    await replaced.evaluate(`localStorage.setItem(${JSON.stringify(key)},${JSON.stringify(malformedUtf16)})`);
    assert.equal((await replaced.command('text.set_run',{expected_revision:0,page_id:1,text_id:1,run_index:0,text:'Phase B'})).changed,true);
    assert.equal(await replaced.evaluate(`localStorage.getItem(${JSON.stringify(key)}) === ${JSON.stringify(malformedUtf16)}`),true);
    assert.equal((await replaced.evaluate('JSON.parse(bondline.host_status())')).recovery_allowed,false);
    report.steps.push('valid supplementary Unicode survives a caption edit; an external malformed normalized counterpart revokes writing');
    await replaced.evaluate(`localStorage.removeItem(${JSON.stringify(key)})`);
    await replaced.call('Target.closeTarget',{targetId:replaced.id});
    page = await newPage();
    assert.deepEqual((await page.command('document.inspect')).document,initial.document);
    const geometry = await page.evaluate('JSON.parse(bondline.canvas_geometry())');
    assert.equal(geometry.pixels_per_point, 1);
    const x = geometry.x + geometry.width/2, y = geometry.y + geometry.height/2;
    await drag(page,x,y);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).document.pages[0].fragments.length === 1`);
    const drawn = await page.command('document.inspect');
    assert.equal(drawn.undo_depth, 1); assert.equal(drawn.file.dirty, true);
    const [fragment] = drawn.document.pages[0].fragments;
    assert.deepEqual(fragment.atoms.map(atom => [atom.element,atom.explicit_hydrogens]), [[6,3],[6,3]]);
    assert.deepEqual(fragment.bonds.map(bond => [bond.begin,bond.end,bond.order]), [[1,2,'single']]);
    await page.screenshot('drawn.png');
    const source = await page.evaluate(`localStorage.getItem(${JSON.stringify(key)})`);
    assert(source?.startsWith('<?xml'));
    await writeFile(path.join(output, 'drawn.cdxml'), source);
    report.steps.push('actual pointer drag created C-C with one registry history entry and real CDXML');
    const formula = await page.command('analysis.formula',{expected_revision:drawn.revision,expected_selection_epoch:drawn.ui_selection?.epoch??0});
    assert.equal(formula.analysis.formula,'C2H6'); assert.equal(formula.analysis.status,'available');
    const properties = await page.command('analysis.properties',{expected_revision:drawn.revision,expected_selection_epoch:drawn.ui_selection?.epoch??0});
    assert(!properties.error,'the actual browser registry must expose the delivered mass query');
    assert.equal(properties.analysis.formula,'C2H6');
    assert(Math.abs(properties.analysis.masses.molecular_weight-30.07)<1e-9);
    assert(Math.abs(properties.analysis.masses.exact_mass_da-30.04695019338)<1e-9);
    report.masses = properties.analysis.masses;
    await click(page,290,22); await page.screenshot('analysis-menu.png');
    await click(page,360,128);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).analysis_window_open === true`);
    await page.screenshot('analysis.png');
    assert.deepEqual((await page.command('document.inspect')).document,drawn.document);
    await page.command('ui.analysis',{expected_revision:drawn.revision,visible:false});
    report.steps.push('real View menu opens the shared Analysis window and reports C2H6 without an edit');
    await shortcut(page, false);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).undo_depth === 0`);
    assert.deepEqual((await page.command('document.inspect')).document, initial.document);
    await shortcut(page, true);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).undo_depth === 1`);
    assert.deepEqual((await page.command('document.inspect')).document, drawn.document);
    assert.equal(await page.evaluate(`localStorage.getItem(${JSON.stringify(key)})`), source);
    report.steps.push('real primary-Z and primary-Shift-Z restored exact document and recovery data');
    assert.equal(await page.evaluate('window.bondlineOfflineReady'), true);
    await page.evaluate("caches.open('bondline-test-unrelated').then(cache=>cache.put('./unrelated-only.txt',new Response('must not serve another cache'))) ");
    await page.call('Network.enable');
    await page.call('Network.emulateNetworkConditions', {offline:true,latency:0,downloadThroughput:0,uploadThroughput:0});
    // CDP page-offline does not disable a worker's independent fetch target.
    localNetworkAvailable = false;
    await page.evaluate('window.bondlineReady=false');
    await page.call('Page.reload'); await page.ready();
    const reopened = await page.command('document.inspect');
    assert.deepEqual(reopened.document, drawn.document); assert.equal(reopened.undo_depth,0);
    assert.equal(reopened.file.path,null);
    assert.equal(await page.evaluate("fetch('./THIRD-PARTY-NOTICES.txt').then(r=>r.arrayBuffer()).then(bytes=>crypto.subtle.digest('SHA-256',bytes)).then(hash=>Array.from(new Uint8Array(hash),n=>n.toString(16).padStart(2,'0')).join(''))"),manifest.files['THIRD-PARTY-NOTICES.txt'].sha256);
    assert.equal(await page.evaluate("fetch('./unrelated-only.txt').then(()=>false,()=>true)"),true,'offline requests must not read unrelated origin caches');
    await page.screenshot('offline-reopened.png');
    report.steps.push('service-worker offline reload reopened the same editable drawing');
    await page.call('Emulation.setDeviceMetricsOverride', {width:1160,height:660,deviceScaleFactor:1,mobile:false});
    await page.evaluate('window.bondlineReady=false');
    await page.call('Page.reload'); await page.ready();
    assert.deepEqual((await page.command('document.inspect')).document,drawn.document);
    assert.equal((await page.evaluate('JSON.parse(bondline.host_status())')).zoom_percent,100);
    // The original View menu remains reachable even when Fit is below the label guard.
    await click(page,290,22); await page.screenshot('small-view-menu.png');
    await click(page,340,65);
    await page.wait('JSON.parse(bondline.host_status()).zoom_percent === null');
    await page.screenshot('small-fit-refusal.png');
    assert.deepEqual((await page.command('document.inspect')).document,drawn.document);
    assert.equal(await page.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),source);
    await click(page,290,22); await click(page,340,44);
    await page.wait('JSON.parse(bondline.host_status()).zoom_percent === 100');
    await page.wait('(() => {try{return !!JSON.parse(bondline.canvas_geometry()).width}catch{return false}})()');
    await page.screenshot('small-readable.png');
    assert.deepEqual((await page.command('document.inspect')).document,drawn.document);
    const small = await page.evaluate('JSON.parse(bondline.canvas_geometry())');
    await drag(page,small.x+small.width/2,small.y+small.height/2+80);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).undo_depth === 1`);
    const smallEdited = await page.command('document.inspect');
    assert.equal(smallEdited.document.pages[0].fragments.length,2);
    await shortcut(page,false);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).undo_depth === 0`);
    assert.deepEqual((await page.command('document.inspect')).document,drawn.document);
    await shortcut(page,true);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).undo_depth === 1`);
    assert.deepEqual((await page.command('document.inspect')).document,smallEdited.document);
    await page.screenshot('small-edited.png');
    await shortcut(page,false);
    await page.wait(`JSON.parse(bondline.execute('{"command":"document.inspect","params":{}}')).undo_depth === 0`);
    assert.equal(await page.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),source);
    report.steps.push('660px offline reload starts readable; visible Fit/Actual and pointer/keyboard history preserve the exact drawing');
    assert.equal((await page.command('document.save_cdxml',{path:'/browser-is-not-a-filesystem.cdxml'})).error.code,'backend_unavailable');
    // Fill this isolated profile's real origin storage, retaining the previous draft.
    const quota = await page.evaluate(`(() => {
        let lo=0,hi=8*1024*1024,failures=0;
        while(lo<hi){const mid=Math.ceil((lo+hi)/2);try{localStorage.setItem('bondline-test-quota','x'.repeat(mid));lo=mid}
            catch(error){if(error.name!=='QuotaExceededError')throw error;failures++;hi=mid-1}}
        return {bytes:lo,failures};
    })()`);
    assert(quota.failures>0,'the browser must actually report its storage quota');
    assert.equal((await page.command('bond.create',edit)).changed,true);
    assert.equal(await page.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),source);
    const full = await page.evaluate('JSON.parse(bondline.host_status())');
    assert.equal(full.unsaved_in_tab,true); assert.match(full.recovery,/could not be saved/);
    await page.evaluate("localStorage.removeItem('bondline-test-quota')");
    await page.command('history.undo');
    assert.equal(await page.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),source);
    report.quota = quota;
    report.steps.push('real storage quota failure preserves the earlier draft and reports unsaved edits');
    const second = await newPage();
    assert.equal((await second.evaluate('JSON.parse(bondline.host_status())')).recovery_allowed,false);
    assert.equal((await second.command('bond.create',edit)).changed,true);
    assert.equal(await second.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),source);
    assert.equal((await second.evaluate('JSON.parse(bondline.host_status())')).unsaved_in_tab,true);
    const secondState = await second.command('document.inspect');
    const secondCanvas = await second.evaluate('JSON.parse(bondline.canvas_geometry())');
    await click(second,secondCanvas.x+secondCanvas.width/2,secondCanvas.y+secondCanvas.height/2+120);
    assert.equal(await second.evaluate('navigator.userActivation.hasBeenActive'),true);
    assert.deepEqual((await second.command('document.inspect')).document,secondState.document);
    const eventStart = second.events.length;
    const attemptedReload = second.call('Page.reload');
    let dialog;
    for (let i=0;i<100&&!dialog;i++) {
        dialog = second.events.slice(eventStart).find(event=>event.method==='Page.javascriptDialogOpening');
        if (!dialog) await delay(50);
    }
    assert.equal(dialog?.params.type,'beforeunload','actual activated tab must protect unrecovered edits');
    await second.call('Page.handleJavaScriptDialog',{accept:false}); await attemptedReload;
    assert.deepEqual((await second.command('document.inspect')).document,secondState.document);
    await second.command('history.undo');
    await second.call('Target.closeTarget',{targetId:second.id});
    report.steps.push('second-tab edit cannot overwrite the writer draft; Cancel on actual unload dialog retains its unsaved document');
    const corrupt = 'preserve externally replaced invalid XML < & β';
    await page.evaluate(`localStorage.setItem(${JSON.stringify(key)},${JSON.stringify(corrupt)})`);
    assert.equal((await page.command('bond.create',edit)).changed,true);
    assert.equal(await page.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),corrupt);
    const conflict = await page.evaluate('JSON.parse(bondline.host_status())');
    assert.equal(conflict.recovery_allowed,false); assert.equal(conflict.unsaved_in_tab,true);
    assert.match(conflict.recovery,/changed elsewhere/);
    await page.command('history.undo'); await page.call('Target.closeTarget',{targetId:page.id});
    const recovery = await newPage();
    assert.equal(await recovery.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),corrupt);
    const bad = await recovery.evaluate('JSON.parse(bondline.host_status())');
    assert.equal(bad.recovery_allowed,false); assert.match(bad.recovery,/preserved/);
    await recovery.command('bond.create',edit);
    assert.equal(await recovery.evaluate(`localStorage.getItem(${JSON.stringify(key)})`),corrupt);
    report.steps.push('external replacement and corrupt startup preserve prior bytes and disable recovery');
    assert(blockedRequests>0,'the owned server must witness rejected worker fallback requests');
    report.local_network_rejected_requests = blockedRequests;
    report.completed = true;
    console.log(JSON.stringify(report));
} catch (error) {
    report.failure = String(error); console.error(String(error)); process.exitCode=1;
    for (const [index, client] of clients.entries()) {
        try {
            await client.screenshot(`failure-${index}.png`);
            report[`failure_page_${index}`] = await client.evaluate('(async()=>({ready:window.bondlineReady,error:window.bondlineStartupError,body:document.body.innerText,workers:await navigator.serviceWorker.getRegistrations().then(rs=>rs.map(r=>({active:r.active?.state,installing:r.installing?.state,waiting:r.waiting?.state}))),cache_keys:await caches.keys()}))()');
            report[`events_${index}`] = client.events;
        } catch {}
    }
} finally {
    await writeFile(path.join(output,'report.json'), JSON.stringify(report,null,2)+'\n');
    await writeFile(path.join(output,'chrome.stderr.log'),stderr);
    for (const client of clients) client.socket.close();
    chrome.kill('SIGTERM'); await delay(300); server.close();
    await rm(profile,{recursive:true,force:true});
}
