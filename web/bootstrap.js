// SPDX-License-Identifier: MIT OR Apache-2.0
// Bootstrap only; the workspace, editing and history live in the shared Rust UI.
import init, {WebHandle} from './bondline_ui.js';
const status = document.getElementById('startup');
try {
    await init();
    const app = new WebHandle();
    // One cooperative writer per origin; the lock is released with the page context.
    let writer = false;
    if (navigator.locks) {
        writer = await new Promise(resolve => {
            navigator.locks.request(app.recovery_key(), {ifAvailable:true}, lock => {
                resolve(Boolean(lock));
                if (lock) return new Promise(() => {});
            }).catch(() => resolve(false));
        });
    }
    await app.start(document.getElementById('bondline'), writer);
    window.bondline = app;
    window.bondlineReady = true;
    status.hidden = true;
    document.getElementById('bondline').focus({preventScroll:true});
    window.addEventListener('beforeunload', event => {
        try {
            if (JSON.parse(app.host_status()).unsaved_in_tab) {
                event.preventDefault();
                event.returnValue = '';
            }
        } catch { /* A destroyed workspace has no live draft. */ }
    });
    if ('serviceWorker' in navigator) {
        window.bondlineOfflineReady = navigator.serviceWorker.register('./sw.js').then(async () => {
            await navigator.serviceWorker.ready;
            if (!navigator.serviceWorker.controller) {
                await new Promise(resolve => navigator.serviceWorker.addEventListener('controllerchange', resolve, {once:true}));
            }
            return true;
        }).catch(() => false);
    }
} catch (error) {
    window.bondlineStartupError = String(error);
    status.textContent = 'Bondline could not start in this browser. Check WebGPU support and reload.';
}
