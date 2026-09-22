import assert from "node:assert/strict";
import test from "node:test";

import { configStore } from "../src/ts/config_store.ts";

test("config storage persists all settings and validates stored values", t => {
  const values = new Map();
  const originalStorage = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: {
      getItem: key => values.get(key) ?? null,
      setItem: (key, value) => values.set(key, value),
    },
  });
  t.after(() => {
    if (originalStorage) Object.defineProperty(globalThis, "localStorage", originalStorage);
    else delete globalThis.localStorage;
  });

  const defaults = { midiVolume: 0.5, pcmVolume: 0.5, enableWasmAot: false, helpDismissed: false, welcomeSeen: false };
  for (const [key, value] of Object.entries(defaults)) {
    assert.equal(configStore.get(key), value);
  }

  const saved = { midiVolume: 0, pcmVolume: 1, enableWasmAot: true, helpDismissed: true, welcomeSeen: true };
  for (const [key, value] of Object.entries(saved)) configStore.set(key, value);
  for (const [key, value] of Object.entries(saved)) assert.equal(configStore.get(key), value);

  configStore.set("midiVolume", 0.37);
  configStore.set("enableWasmAot", false);
  assert.equal(configStore.get("midiVolume"), 0.37);
  assert.equal(configStore.get("enableWasmAot"), false);
  assert.equal(configStore.get("helpDismissed"), true);

  for (const [key, fallback] of Object.entries(defaults)) {
    const invalid = typeof fallback === "number" ? ["-1", "2", '"0.5"', "true"] : ['"true"', "1"];
    for (const value of [...invalid, "null", "{}", "[]", "invalid JSON"]) {
      values.set(`wie_config_${key}`, value);
      assert.equal(configStore.get(key), fallback, `${key}: ${value}`);
    }
  }

  localStorage.getItem = () => { throw new Error("Storage unavailable"); };
  assert.equal(configStore.get("enableWasmAot"), false);
});
