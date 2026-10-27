import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";

test("Android IPC keeps its key in the main frame and preserves message data", async () => {
  const key = "test-main-frame-invoke-key";
  const source = (await readFile(new URL("../src/android_ipc.js", import.meta.url), "utf8"))
    .replaceAll("__INVOKE_KEY__", JSON.stringify(key));
  const received = [];
  const main = vm.createContext({ receive: value => received.push(JSON.parse(value)) });
  vm.runInContext(`
    window = { __TAURI_INTERNALS__: {}, ipc: { postMessage: receive } };
    window.top = window;
  `, main);
  vm.runInContext(source, main);
  assert.ok(!vm.runInContext("window.__TAURI_INTERNALS__.postMessage.toString()", main).includes(key));

  vm.runInContext(`window.__TAURI_INTERNALS__.postMessage({
    cmd: "start_app", callback: 3, error: 4,
    payload: {
      id: "sample",
      events: { __TAURI_TO_IPC_KEY__() { return "__CHANNEL__:7"; } },
      bytes: new Uint8Array([0, 127, 255]),
      buffer: new Uint8Array([9, 10]).buffer,
      metadata: new Map([["nested", new Map([["value", 8]])]]),
      empty: null
    },
    options: { headers: { "Example": "preserved" }, customProtocolIpcBlocked: false }
  });`, main);
  const message = received.shift();
  assert.equal(message.cmd, "start_app");
  assert.equal(message.callback, 3);
  assert.equal(message.error, 4);
  assert.equal(message.__TAURI_INVOKE_KEY__, key);
  assert.deepEqual(message.payload, {
    id: "sample", events: "__CHANNEL__:7", bytes: [0, 127, 255],
    buffer: [9, 10], metadata: { nested: { value: 8 } }, empty: null,
  });
  assert.equal(message.options.headers.Example, "preserved");
  assert.equal(message.options.customProtocolIpcBlocked, true);

  vm.runInContext(`window.__TAURI_INTERNALS__.postMessage({
    cmd: "plugin:__TAURI_CHANNEL__|fetch", callback: 11, error: 12,
    payload: null, options: { headers: { "Tauri-Channel-Id": "13" } }
  });`, main);
  const channel = received.shift();
  assert.equal(channel.payload, null);
  assert.equal(channel.options.headers["Tauri-Channel-Id"], "13");

  const child = vm.createContext({ receive: value => received.push(value) });
  vm.runInContext(`
    window = { top: {}, __TAURI_INTERNALS__: {}, ipc: { postMessage: receive } };
  `, child);
  vm.runInContext(source, child);
  assert.equal(vm.runInContext('Object.hasOwn(window.__TAURI_INTERNALS__, "postMessage")', child), false);
  assert.ok(!vm.runInContext("JSON.stringify(window)", child).includes(key));
  assert.equal(received.length, 0);
});
