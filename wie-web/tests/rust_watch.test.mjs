import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { EventEmitter } from "node:events";
import { syncBuiltinESMExports } from "node:module";
import path from "node:path";
import test from "node:test";

import commonConfig from "../webpack.config.common.ts";

test("Rust watch tracks workspace dependencies without watching unrelated artifacts", async t => {
  const root = path.resolve(import.meta.dirname, "../..");
  const callbacks = {};
  const compiler = {
    options: { mode: "development" },
    hooks: Object.fromEntries(["watchRun", "beforeCompile", "afterCompile"].map(hook => [hook, {
      tap: (_name, callback) => { callbacks[hook] = callback; },
      tapPromise: (_name, callback) => { callbacks[hook] = callback; },
    }])),
  };
  const plugin = commonConfig("development").plugins.find(plugin => plugin.constructor.name === "WasmPackPlugin");
  plugin.apply(compiler);

  let builds = 0;
  let exitCode = 0;
  t.mock.method(childProcess, "spawn", (command, args) => {
    assert.equal(command, "wasm-pack");
    assert.deepEqual(args, ["build", path.join(root, "wie-web"), "--target", "bundler", "--dev"]);
    builds += 1;
    const child = new EventEmitter();
    queueMicrotask(() => child.emit("exit", exitCode));
    return child;
  });
  syncBuiltinESMExports();
  t.after(() => {
    t.mock.restoreAll();
    syncBuiltinESMExports();
  });

  callbacks.watchRun();
  await callbacks.beforeCompile();
  assert.equal(builds, 1);

  const compilation = { contextDependencies: new Set(), fileDependencies: new Set() };
  callbacks.afterCompile(compilation);
  for (const dir of ["wie-web/src/rust", "wie-core-arm/src", "wie-core-arm-wasm/src", "wie-arm-jit-types/src", "wie-jvm-support/src"]) {
    assert.ok(compilation.contextDependencies.has(path.join(root, dir)), dir);
  }
  for (const file of ["Cargo.toml", "Cargo.lock", "wie-web/Cargo.toml", "wie-core-arm-wasm/Cargo.toml", "wie-arm-jit-types/Cargo.toml"]) {
    assert.ok(compilation.fileDependencies.has(path.join(root, file)), file);
  }
  for (const file of ["target/debug/unused.rs", "node_modules/unused.rs", "docs/unused.rs", "wie-app/src/lib.rs", "test-utils/src/lib.rs", "wie-web/src/ts/settings.ts"]) {
    const absolute = path.join(root, file);
    assert.ok(![...compilation.contextDependencies].some(dir => absolute === dir || absolute.startsWith(dir + path.sep)), file);
    compiler.modifiedFiles = new Set([absolute]);
    callbacks.watchRun();
    await callbacks.beforeCompile();
    assert.equal(builds, 1, file);
  }

  for (const file of ["wie-core-arm-wasm/src/lib.rs", "wie-arm-jit-types/src/lib.rs", "Cargo.toml", "Cargo.lock", "wie-core-arm/Cargo.toml"]) {
    compiler.modifiedFiles = new Set([path.join(root, file)]);
    callbacks.watchRun();
    const previous = builds;
    await callbacks.beforeCompile();
    assert.equal(builds, previous + 1, file);
  }

  compiler.modifiedFiles = new Set();
  compiler.removedFiles = new Set([path.join(root, "wie-core-arm-wasm/src/removed.rs")]);
  callbacks.watchRun();
  const previous = builds;
  exitCode = 1;
  await assert.rejects(callbacks.beforeCompile(), /wasm-pack exited with code 1/);
  compiler.removedFiles.clear();
  callbacks.watchRun();
  exitCode = 0;
  await callbacks.beforeCompile();
  assert.equal(builds, previous + 2);
});
