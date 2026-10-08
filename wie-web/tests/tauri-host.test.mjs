import assert from "node:assert/strict";
import { afterEach, test } from "node:test";

import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";

import { TauriHost } from "../src/ts/backend/tauri-host.ts";

Object.defineProperty(globalThis, "window", { value: { crypto: globalThis.crypto }, configurable: true });
afterEach(clearMocks);

test("host preserves storage ordering and drains accepted requests before stopping", async () => {
  const calls = [];
  const write = Promise.withResolvers();
  mockIPC(async (command, args) => {
    assert.equal(args.sessionId, 7);
    calls.push(command);
    if (command === "guest_storage") return write.promise;
  });
  const host = new TauriHost(7, () => assert.fail("unexpected host error"));
  const pending = host.storage({ op: "fileWrite", aid: "app", path: "save", offset: 2, data: [3, 4] });
  const stopped = host.stop();
  await assert.rejects(host.storage({ op: "fileSize", aid: "app", path: "save" }), /stopped/);
  await new Promise(resolve => setImmediate(resolve));
  assert.deepEqual(calls, ["guest_storage"]);
  write.resolve(2);
  assert.equal(await pending, 2);
  await stopped;
  await host.stop();
  assert.deepEqual(calls, ["guest_storage", "stop_game"]);
});

test("host converts audio events and reports failed requests without preventing cleanup", async () => {
  const commands = [];
  const errors = [];
  mockIPC((command, args) => {
    commands.push({ command, args });
    if (command === "guest_storage") throw new Error("storage unavailable");
  });
  const host = new TauriHost(9, error => errors.push(error));
  host.audio.play(3, 20, [[0, "midi", new Uint8Array([0x90, 60, 100])], [10, "wave", 1, 8000, new Int16Array([1, -2])]], false);
  host.audio.stop(3);
  await assert.rejects(host.storage({ op: "dbOpen", pid: "pid", name: "save" }), /storage unavailable/);
  await assert.rejects(host.stop(), /storage unavailable/);
  assert.equal(errors.length, 1);
  assert.deepEqual(commands, [
    { command: "guest_audio", args: { sessionId: 9, command: { type: "play", handle: 3, duration: 20, events: [
      { time: 0, kind: "midi", data: [0x90, 60, 100] },
      { time: 10, kind: "wave", channels: 1, samplingRate: 8000, samples: [1, -2] },
    ], repeat: false } } },
    { command: "guest_audio", args: { sessionId: 9, command: { type: "stop", handle: 3 } } },
    { command: "guest_storage", args: { sessionId: 9, request: { op: "dbOpen", pid: "pid", name: "save" } } },
    { command: "stop_game", args: { sessionId: 9 } },
  ]);

  let stopped = false;
  mockIPC(command => {
    if (command === "guest_audio") throw new Error("audio unavailable");
    if (command === "stop_game") stopped = true;
  });
  const audioErrors = [];
  const audioHost = new TauriHost(10, error => audioErrors.push(error));
  audioHost.audio.play(1, 10, [[0, "midi", new Uint8Array([0x90, 60, 100])]], false);
  await new Promise(resolve => setImmediate(resolve));
  await assert.rejects(audioHost.stop(), /audio unavailable/);
  assert.equal(audioErrors.length, 1);
  assert.equal(stopped, true);
});

test("host clock excludes suspended time and never moves backwards", context => {
  let time = 100;
  context.mock.method(performance, "now", () => time);
  const host = new TauriHost(1, () => assert.fail("unexpected host error"));
  host.lifecycle({ type: "lifecycle", suspended: false, guestTimeMs: 1000 });
  time += 25;
  assert.equal(host.now(), 1025);
  host.lifecycle({ type: "lifecycle", suspended: true, guestTimeMs: 1025 });
  time += 60_000;
  assert.equal(host.now(), 1025);
  host.lifecycle({ type: "lifecycle", suspended: false, guestTimeMs: 1025 });
  time += 10;
  assert.equal(host.now(), 1035);
  host.lifecycle({ type: "lifecycle", suspended: true, guestTimeMs: 1030 });
  assert.equal(host.now(), 1035);
});
