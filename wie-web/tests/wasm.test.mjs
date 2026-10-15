import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";

import ts from "typescript";

function loadSession(failure, duringConstruction = false) {
  const frames = [];
  const events = [];
  const calls = { free: 0, dispose: 0 };
  const panic = {};
  const load = (filename, exports, imports = {}) => {
    const source = readFileSync(new URL(filename, import.meta.url), "utf8");
    const compiled = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS } });
    runInNewContext(compiled.outputText, {
      exports, Error, require: name => imports[name],
      document: { getElementById: () => ({ width: 0, height: 0 }) },
      requestAnimationFrame: callback => frames.push(callback),
      cancelAnimationFrame: frame => { frames[frame - 1] = undefined; },
    });
    return exports;
  };
  load("../src/ts/panic.ts", panic);
  const fail = () => {
    if (failure === "panic") {
      panic.recordPanic("original Rust panic");
      throw new WebAssembly.RuntimeError("unreachable");
    }
    throw failure;
  };
  class AudioPlayer {
    dispose() { calls.dispose++; }
  }
  class WieWeb {
    constructor(filename, archive, canvas, font, audio) {
      this.audio = audio;
      if (duringConstruction) fail();
    }
    update() { fail(); }
    free() {
      calls.free++;
      if (panic.rustPanic) throw new Error("attempted to take ownership of Rust value while it was borrowed");
      this.audio.dispose();
    }
  }
  const { startWasmSession } = load("../src/ts/backend/wasm.ts", {}, {
    "@pkg": { WieWeb }, "../midi": { AudioPlayer }, "../panic": panic,
  });
  return {
    start: () => startWasmSession("app.jar", new Uint8Array(), new Uint8Array(), event => events.push(event)),
    frames, events, calls, panic,
  };
}

test("returned errors preserve the exception and release the emulator once", async () => {
  const error = new Error("guest failure");
  const runtime = loadSession(error);
  const session = runtime.start();
  runtime.frames[0]();
  runtime.frames[1]();
  const event = runtime.events.find(event => event.type === "error");
  assert.equal(event.cause, error);
  await session.stop();
  await session.stop();
  assert.deepEqual(runtime.calls, { free: 1, dispose: 1 });
});

test("panics preserve the original message without reentering borrowed Rust state", async () => {
  const runtime = loadSession("panic");
  const session = runtime.start();
  runtime.frames[0]();
  runtime.frames[1]();
  const event = runtime.events.find(event => event.type === "error");
  assert.equal(event.cause, runtime.panic.rustPanic);
  assert.equal(event.cause.name, "RustPanic");
  assert.equal(event.cause.message, "original Rust panic");
  await session.stop();
  await session.stop();
  assert.throws(runtime.start, error => error === event.cause);
  assert.deepEqual(runtime.calls, { free: 0, dispose: 1 });
});

test("constructor panics release host audio and retain the Rust panic", () => {
  const runtime = loadSession("panic", true);
  assert.throws(runtime.start, error => error === runtime.panic.rustPanic);
  assert.deepEqual(runtime.calls, { free: 0, dispose: 1 });
});
