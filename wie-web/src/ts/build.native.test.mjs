import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import webpack from "webpack";

import commonConfig from "../../webpack.config.common.ts";

test("native build contains controls without the browser runtime or media", async () => {
  const output = mkdtempSync(path.join(tmpdir(), "wie-native-ui-"));
  try {
    const config = commonConfig("production", true);
    assert.notEqual(config.output.path, commonConfig("production").output.path);
    const compiler = webpack({ ...config, mode: "production", output: { ...config.output, path: output } });
    const stats = await new Promise((resolve, reject) => {
      compiler.run((error, result) => {
        compiler.close(closeError => {
          if (error || closeError) reject(error ?? closeError);
          else if (result.hasErrors()) reject(new Error(result.toString({ all: false, errors: true })));
          else resolve(result);
        });
      });
    });
    const modules = [...stats.compilation.modules].map(module => module.identifier()).join("\n");
    assert.doesNotMatch(modules, /backend\.browser|app_library_store|config_store|audio-worker|arm-compiler|midi\.ts|wie_web|spessasynth|@sentry/);
    const assets = Object.keys(stats.compilation.assets);
    assert(!assets.some(name => /\.(wasm|sf2|sf3|dls)$/i.test(name)));
    const html = readFileSync(path.join(output, "index.html"), "utf8");
    assert.doesNotMatch(html, /<canvas|adsbygoogle|googletagmanager|enable-wasm-aot|<script[^>]+src=["']https?:/);
    assert.match(html, /id="player-status"/);
    assert.match(html, /id="player-warning"/);
    assert.match(html, /data-key="OK"/);
    const javascript = assets.filter(name => name.endsWith(".js")).map(name => readFileSync(path.join(output, name), "utf8")).join("\n");
    assert.doesNotMatch(javascript, /WebAssembly|AudioContext|AudioWorklet|GeneralUser|spessasynth/);
  } finally {
    rmSync(output, { recursive: true, force: true });
  }
});
