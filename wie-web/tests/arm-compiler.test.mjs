import assert from "node:assert/strict";
import { setImmediate } from "node:timers/promises";
import test from "node:test";

import { compileArm } from "../src/ts/arm-compiler.ts";

test("timed-out preparation never resumes a freed warmup frame", async t => {
    for (const phase of ["compile", "instantiate"]) {
        await t.test(phase, async t => {
            t.mock.timers.enable({ apis: ["setTimeout"] });
            t.mock.method(console, "info", () => {});
            const started = Promise.withResolvers();
            const pending = Promise.withResolvers();
            const bytes = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]);
            const module = new WebAssembly.Module(bytes);
            let calls = 0;
            const dispatch = () => { calls++; return 3; };
            const instance = { exports: { dispatch } };
            t.mock.method(WebAssembly, "compile", async () => {
                if (phase !== "compile") return module;
                started.resolve();
                return pending.promise;
            });
            t.mock.method(WebAssembly, "instantiate", async () => {
                if (phase !== "instantiate") return instance;
                started.resolve();
                return pending.promise;
            });
            const artifact = { bytes, manifest: new Uint8Array() };
            const imports = { wie: { memory: new WebAssembly.Memory({ initial: 1 }) } };
            const preparation = compileArm(artifact, undefined, undefined, imports, 16, 1, performance.now() + 1000);
            const rejected = assert.rejects(preparation, /preparation timed out/);
            await started.promise;
            t.mock.timers.tick(1001);
            await rejected;
            pending.resolve(phase === "compile" ? module : instance);
            await setImmediate();
            assert.equal(calls, 0);

            t.mock.method(WebAssembly, "compile", async () => module);
            t.mock.method(WebAssembly, "instantiate", async () => instance);
            assert.equal(await compileArm(artifact, undefined, undefined, imports, 16, 1, performance.now() + 1000), dispatch);
            assert.equal(calls, 1);
        });
    }
});
