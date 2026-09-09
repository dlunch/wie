import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { EventEmitter } from "node:events";
import { syncBuiltinESMExports } from "node:module";
import path from "node:path";
import test from "node:test";

import webpack from "webpack";

import commonConfig from "./webpack.config.common.ts";

test("source directory watch events rebuild both Wasm packages", async t => {
  const builds = [];
  const spawn = t.mock.method(childProcess, "spawn", (command, args) => {
    assert.equal(command, "wasm-pack");
    builds.push(args[1]);
    const child = new EventEmitter();
    queueMicrotask(() => child.emit("exit", 0));
    return child;
  });
  syncBuiltinESMExports();
  const plugin = commonConfig("development").plugins.find(plugin => plugin.constructor.name === "WasmPackPlugin");
  const compiler = webpack({ mode: "development", plugins: [plugin] });
  const packages = [path.resolve(import.meta.dirname, "../wie-arm-wasm/compiler"), import.meta.dirname];
  try {
    await compiler.hooks.beforeCompile.promise({});
    assert.deepEqual(builds.splice(0), packages);
    for (const changed of [
      "../wie-arm-wasm/src",
      "../wie-arm-jit/src",
      "../wie-core-arm/src",
      "../wie-arm-wasm/compiler/src",
      "./src/rust",
      "../wie-arm-wasm/src/bootstrap.js",
      "../wie-arm-jit/src/lib.rs",
      "../Cargo.lock",
    ]) {
      compiler.modifiedFiles = new Set([path.resolve(import.meta.dirname, changed)]);
      await compiler.hooks.watchRun.promise(compiler);
      await compiler.hooks.beforeCompile.promise({});
      assert.deepEqual(builds.splice(0), packages, changed);
    }
    compiler.modifiedFiles = new Set([path.resolve(import.meta.dirname, "../wie-arm-jit/src-unrelated/file.rs")]);
    await compiler.hooks.watchRun.promise(compiler);
    await compiler.hooks.beforeCompile.promise({});
    assert.deepEqual(builds, []);
  } finally {
    await new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
    spawn.mock.restore();
    syncBuiltinESMExports();
  }
});
