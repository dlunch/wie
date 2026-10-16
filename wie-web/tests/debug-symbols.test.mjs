import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const webDir = fileURLToPath(new URL("../", import.meta.url));

test("release artifacts retain debug IDs without distributing debug files", async () => {
  const dist = path.join(webDir, "dist");
  const files = await readdir(dist, { recursive: true });
  assert.equal(files.some(file => file.endsWith(".map") || file.endsWith(".debug.wasm")), false);

  const wasmFiles = files.filter(file => file.endsWith(".wasm"));
  assert.equal(wasmFiles.length, 1);
  const deployed = await readFile(path.join(dist, wasmFiles[0]));
  assert.deepEqual(deployed, await readFile(path.join(webDir, "pkg/wie_web_bg.wasm")));
  const stripped = await WebAssembly.compile(deployed);
  const debug = await WebAssembly.compile(await readFile(path.join(webDir, "../target/sentry/wie_web.debug.wasm")));
  const buildIds = WebAssembly.Module.customSections(stripped, "build_id");
  assert.equal(buildIds.length, 1);
  assert.ok(buildIds[0].byteLength > 0);
  assert.deepEqual(buildIds, WebAssembly.Module.customSections(debug, "build_id"));
  for (const section of [".debug_info", ".debug_line"]) {
    assert.equal(WebAssembly.Module.customSections(stripped, section).length, 0);
    assert.equal(WebAssembly.Module.customSections(debug, section).length, 1);
  }

  const bundles = files.filter(file => file.startsWith(`assets${path.sep}js${path.sep}`) && file.endsWith(".js"));
  assert.ok(bundles.length > 0);
  for (const bundle of bundles) {
    assert.match(await readFile(path.join(dist, bundle), "utf8"), /_sentryDebugIds/);
  }
});
