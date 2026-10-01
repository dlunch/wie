import assert from "node:assert/strict";
import { afterEach, test } from "node:test";

import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";

import { backend } from "./backend.native.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(() => clearMocks());

test("native library and settings use IPC and decode icon bytes", async () => {
  const app = { id: "test", title: "Test", filename: "test.jar", addedAt: 123, icon: [1, 2, 3] };
  const settings = { midiVolume: 0.5, pcmVolume: 0.3, helpDismissed: true, welcomeSeen: true };
  const requests = [];
  mockIPC((command, args) => {
    requests.push([command, args]);
    switch (command) {
      case "list_apps": return [app];
      case "import_app": return app;
      case "read_settings": return settings;
      case "write_settings":
      case "delete_app": return;
      default: throw new Error(`Unexpected command: ${command}`);
    }
  });

  const [listed] = await backend.listApps();
  assert.equal(listed.id, app.id);
  assert.deepEqual(new Uint8Array(await listed.icon.arrayBuffer()), Uint8Array.of(1, 2, 3));
  const imported = await backend.importApp(new File([Uint8Array.of(4, 5)], "test.jar"));
  assert.equal(imported.title, app.title);
  assert.deepEqual(requests.find(([command]) => command === "import_app")[1], { filename: "test.jar", bytes: [4, 5] });
  assert.deepEqual(await backend.readSettings(), settings);
  await backend.writeSettings({ ...settings, pcmVolume: 0.8 });
  assert.deepEqual(requests.find(([command]) => command === "write_settings")[1], { settings: { ...settings, pcmVolume: 0.8 } });
  await backend.deleteApp(app.id);
  assert.deepEqual(requests.at(-1), ["delete_app", { id: "test" }]);
});

test("session events are subscribed before launch and commands finish in order", async () => {
  const events = [];
  const commands = [];
  let channel;
  let releaseInput;
  const inputBlocked = new Promise(resolve => { releaseInput = resolve; });
  mockIPC(async (command, args) => {
    if (command === "start_game") {
      assert.equal(args.id, "test");
      channel = args.events;
      channel.onmessage({ type: "warning", message: "MIDI unavailable" });
      channel.onmessage({ type: "ready" });
      return 42;
    }
    commands.push([command, args]);
    if (command === "key_event" && args.pressed) await inputBlocked;
  });

  const session = await backend.startGame("test", event => events.push(event));
  assert.deepEqual(events, [{ type: "warning", message: "MIDI unavailable" }, { type: "ready" }]);
  const pressed = session.key("1", true);
  const released = session.key("1", false);
  const reset = session.releaseKeys();
  const stopped = session.stop();
  await Promise.resolve();
  assert.equal(commands.length, 1);
  releaseInput();
  await Promise.all([pressed, released, reset, stopped, session.stop()]);
  assert.deepEqual(commands, [
    ["key_event", { sessionId: 42, key: "1", pressed: true }],
    ["key_event", { sessionId: 42, key: "1", pressed: false }],
    ["release_keys", { sessionId: 42 }],
    ["stop_game", { sessionId: 42 }],
  ]);
  channel.onmessage({ type: "ready" });
  await session.key("1", true);
  assert.equal(events.length, 2);
  assert.equal(commands.length, 4);
});

test("an input rejection does not prevent session cleanup", async () => {
  const commands = [];
  mockIPC((command) => {
    commands.push(command);
    if (command === "start_game") return 7;
    if (command === "key_event") throw new Error("Input failed");
  });
  const session = await backend.startGame("test", () => {});
  await assert.rejects(session.key("UP", true), /Input failed/);
  await session.stop();
  assert.deepEqual(commands, ["start_game", "key_event", "stop_game"]);
});
