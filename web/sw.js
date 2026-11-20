// SPDX-License-Identifier: MIT OR Apache-2.0
// The packager replaces this token with the complete asset-set hash.
const PREFIX = `bondline:${self.registration.scope}:`;
const CACHE = PREFIX + '__BUNDLE_HASH__';
const ASSETS = ['./', './index.html', './bootstrap.js', './bondline_ui.js', './bondline_ui_bg.wasm', './THIRD-PARTY-NOTICES.txt'];
self.addEventListener('install', event => {
    event.waitUntil(caches.open(CACHE).then(cache => cache.addAll(ASSETS)));
});
self.addEventListener('activate', event => {
    event.waitUntil(caches.keys().then(keys => Promise.all(keys.filter(key => key.startsWith(PREFIX) && key !== CACHE).map(key => caches.delete(key)))).then(() => self.clients.claim()));
});
self.addEventListener('fetch', event => {
    if (event.request.method === 'GET' && new URL(event.request.url).origin === self.location.origin) {
        event.respondWith(caches.open(CACHE).then(cache => cache.match(event.request)).then(cached => cached || fetch(event.request)));
    }
});
