const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');
const ts = require('typescript');

const deferred = () => {
  let resolve;
  let reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};

async function harness() {
  const creation = deferred();
  const events = [];
  const frames = new Map();
  const document = new EventTarget();
  const window = new EventTarget();
  let canvas;
  let nextFrame = 0;
  let freed = 0;
  let disposed = 0;
  let updates = 0;
  let observed = 0;
  let onError;
  let createArgs;
  const redraws = [];
  const emulator = {
    free() { freed++; },
    update() { updates++; return false; },
    redraw(...args) { redraws.push(args); return false; },
    is_preparing() { return false; },
    is_exited() { return false; },
    key_up() {},
    key_down() {},
  };
  document.hidden = false;
  document.createElement = () => Object.assign(new EventTarget(), {
    width: 300, height: 150, id: '',
    getBoundingClientRect: () => ({ width: 180, height: 240 }),
    remove() { if (canvas === this) canvas = undefined; },
  });
  document.getElementById = id => id === 'canvas' ? canvas : { prepend(value) { canvas = value; } };
  window.devicePixelRatio = 2;
  window.matchMedia = () => new EventTarget();
  const context = vm.createContext({
    console, document, window, AbortController, Event, DOMException, Uint8Array,
    requestAnimationFrame(callback) { frames.set(++nextFrame, callback); return nextFrame; },
    cancelAnimationFrame(id) { frames.delete(id); },
    ResizeObserver: class {
      connected = false;
      observe() { this.connected = true; observed++; }
      disconnect() { if (this.connected) observed--; this.connected = false; }
    },
  });
  const source = fs.readFileSync(path.join(__dirname, '../src/ts/backend/wasm.ts'), 'utf8');
  const module = new vm.SourceTextModule(ts.transpile(source, { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext }), { context });
  const bindings = {
    '@pkg': { WieWeb: { create(...args) { createArgs = args; onError = args[6].onError; return creation.promise; } } },
    '../midi': { AudioPlayer: class { dispose() { disposed++; } } },
    '../panic': { rustPanic: undefined },
  };
  await module.link(specifier => new vm.SyntheticModule(Object.keys(bindings[specifier]), function () {
    for (const [name, value] of Object.entries(bindings[specifier])) this.setExport(name, value);
  }, { context }));
  await module.evaluate();
  return {
    creation, events, frames, emulator, redraws, document,
    start(signal, suspended = () => false, check = () => true) {
      return module.namespace.startWasmSession('fixture.jar', new Uint8Array(), new Uint8Array(),
        event => events.push(event), signal, 'nearest', undefined, suspended, check);
    },
    error(error) { onError(error); },
    frame() {
      const callbacks = [...frames.values()];
      frames.clear();
      for (const callback of callbacks) callback();
    },
    state: () => ({ freed, disposed, updates, observed, canvas, createArgs }),
  };
}

test('abort during GPU creation frees the late object without activating or publishing', async () => {
  const h = await harness();
  const abort = new AbortController();
  const starting = h.start(abort.signal);
  abort.abort();
  h.creation.resolve(h.emulator);
  assert.equal(await starting, undefined);
  assert.equal(h.state().freed, 1);
  assert.equal(h.state().observed, 0);
  assert.equal(h.state().canvas, undefined);
  assert.equal(h.frames.size, 0);
  assert.equal(h.events.length, 0);
});

for (const initiallySuspended of [false, true]) {
  test(`startup uses current suspension instead of its ${initiallySuspended} snapshot`, async () => {
    const h = await harness();
    const abort = new AbortController();
    let suspended = initiallySuspended;
    const starting = h.start(abort.signal, () => suspended);
    suspended = !suspended;
    h.creation.resolve(h.emulator);
    const session = await starting;
    h.frame();
    h.frame();
    assert.equal(h.state().updates, suspended ? 0 : 1);
    assert.equal(h.events[0].suspended, suspended);
    assert.equal(h.state().createArgs[6].scaleFilter, 'nearest');
    await session.stop();
    assert.equal(h.state().freed, 1);
    assert.equal(h.state().observed, 0);
    assert.equal(h.frames.size, 0);
  });
}

test('paused resize redraws retained output, and GPU errors still terminate', async () => {
  const h = await harness();
  const abort = new AbortController();
  const starting = h.start(abort.signal, () => true);
  h.creation.resolve(h.emulator);
  const session = await starting;
  h.frame();
  assert.deepEqual(h.redraws.at(-1), [360, 480, true]);
  assert.equal(h.state().updates, 0);
  h.error(new Error('GPU device lost'));
  assert.equal(h.events.at(-1).type, 'error');
  assert.equal(h.state().freed, 1);
  assert.equal(h.state().observed, 0);
  assert.equal(h.frames.size, 0);
  const eventCount = h.events.length;
  h.error(new Error('late error'));
  await session.stop();
  assert.equal(h.events.length, eventCount);
});

test('terminal host state after GPU creation prevents activation', async () => {
  const h = await harness();
  let alive = true;
  const starting = h.start(new AbortController().signal, () => false, () => alive);
  alive = false;
  h.creation.resolve(h.emulator);
  assert.equal(await starting, undefined);
  assert.equal(h.state().freed, 1);
  assert.equal(h.state().observed, 0);
  assert.equal(h.frames.size, 0);
  assert.equal(h.events.length, 0);
});

test('startup rejection disposes audio and releases output observers', async () => {
  const h = await harness();
  const starting = h.start(new AbortController().signal);
  h.creation.reject(new Error('No compatible presentation adapter'));
  await assert.rejects(starting, /No compatible presentation adapter/);
  assert.equal(h.state().disposed, 1);
  assert.equal(h.state().observed, 0);
  assert.equal(h.state().canvas, undefined);
});

async function tauriHarness(runtime) {
  const acquired = deferred();
  const gpu = deferred();
  const calledWasm = deferred();
  const calls = [];
  const events = [];
  let channel;
  let wasmArgs;
  const context = vm.createContext({
    console, URL, Blob, Uint8Array, performance,
    fetch: async () => ({ ok: true, arrayBuffer: async () => new ArrayBuffer(0) }),
  });
  const bindings = {
    '@tauri-apps/api/core': {
      Channel: class { constructor(callback) { channel = callback; } },
      invoke: async (command, args) => {
        calls.push([command, args]);
        if (command === 'runtime_kind') return runtime;
        if (command === 'start_app') return acquired.promise;
        if (command === 'read_settings') return { scaleFilter: 'nearest' };
      },
    },
    './wasm': {
      startWasmSession: async (...args) => {
        wasmArgs = args;
        calledWasm.resolve();
        await gpu.promise;
        if (args[4].aborted || !args[8]()) return;
        return {
          key() {}, releaseKeys() {}, suspend() {},
          async stop() { calls.push(['wasm_stop']); },
        };
      },
    },
  };
  const modules = new Map();
  const load = async specifier => {
    if (modules.has(specifier)) return modules.get(specifier);
    let module;
    if (bindings[specifier]) {
      module = new vm.SyntheticModule(Object.keys(bindings[specifier]), function () {
        for (const [name, value] of Object.entries(bindings[specifier])) this.setExport(name, value);
      }, { context });
    } else {
      const source = fs.readFileSync(path.join(__dirname, '../src/ts/backend', `${specifier}.ts`), 'utf8');
      module = new vm.SourceTextModule(ts.transpile(source, { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext }), {
        context,
        initializeImportMeta(meta) { meta.url = 'file:///wie/backend/tauri.ts'; },
        importModuleDynamically: async specifier => {
          const loaded = await load(specifier);
          await loaded.evaluate();
          return loaded;
        },
      });
    }
    modules.set(specifier, module);
    await module.link(load);
    return module;
  };
  const module = await load('./tauri');
  await module.evaluate();
  const backend = await module.namespace.initializeWie();
  return {
    acquired, gpu, calledWasm, calls, events,
    start: signal => backend.startApp('app', event => events.push(event), signal),
    event: event => channel(event),
    args: () => wasmArgs,
  };
}

test('iOS abort wakes the clock wait and stops its acquired host', async () => {
  const h = await tauriHarness('wasm');
  const abort = new AbortController();
  const starting = h.start(abort.signal);
  h.acquired.resolve({ sessionId: 7, filename: 'fixture.jar', bytes: [] });
  await new Promise(resolve => setImmediate(resolve));
  abort.abort();
  assert.equal(await starting, undefined);
  assert.equal(h.calls.filter(([name]) => name === 'stop_app').length, 1);
  assert.equal(h.events.length, 0);
});

test('iOS terminal startup events clean the host without publishing WASM', async () => {
  const h = await tauriHarness('wasm');
  const starting = h.start(new AbortController().signal);
  h.event({ type: 'stopped', failed: true });
  h.acquired.resolve({ sessionId: 7, filename: 'fixture.jar', bytes: [] });
  assert.equal(await starting, undefined);
  assert.equal(h.calls.filter(([name]) => name === 'stop_app').length, 1);
  assert.equal(h.events.length, 1);
});

test('iOS supplies native filter and live lifecycle, stopping WASM before host', async () => {
  const h = await tauriHarness('wasm');
  const starting = h.start(new AbortController().signal);
  h.event({ type: 'lifecycle', suspended: false, guestTimeMs: 0 });
  h.acquired.resolve({ sessionId: 7, filename: 'fixture.jar', bytes: [] });
  await h.calledWasm.promise;
  assert.equal(h.args()[5], 'nearest');
  h.event({ type: 'lifecycle', suspended: true, guestTimeMs: 10 });
  assert.equal(h.args()[7](), true);
  h.gpu.resolve();
  const session = await starting;
  await session.stop();
  assert.deepEqual(h.calls.slice(-2).map(([name]) => name), ['wasm_stop', 'stop_app']);
});

test('iOS terminal event during GPU creation prevents publication and stops host', async () => {
  const h = await tauriHarness('wasm');
  const starting = h.start(new AbortController().signal);
  h.event({ type: 'lifecycle', suspended: false, guestTimeMs: 0 });
  h.acquired.resolve({ sessionId: 7, filename: 'fixture.jar', bytes: [] });
  await h.calledWasm.promise;
  h.event({ type: 'stopped', failed: false });
  assert.equal(h.args()[8](), false);
  h.gpu.resolve();
  assert.equal(await starting, undefined);
  assert.equal(h.calls.filter(([name]) => name === 'stop_app').length, 1);
});

test('native session acquired after abort is stopped without late events', async () => {
  const h = await tauriHarness('native');
  const abort = new AbortController();
  const starting = h.start(abort.signal);
  abort.abort();
  h.event({ type: 'ready' });
  h.acquired.resolve(7);
  assert.equal(await starting, undefined);
  assert.equal(h.calls.filter(([name]) => name === 'stop_app').length, 1);
  assert.equal(h.events.length, 0);
});
