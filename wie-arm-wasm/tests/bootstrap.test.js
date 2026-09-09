import assert from "node:assert/strict";
import test from "node:test";

import { createCompilerWorker } from "../src/bootstrap.js";

test("the backend creates its own module worker and propagates creation failures", () => {
    const previous = globalThis.Worker;
    try {
        globalThis.Worker = class {
            constructor(url, options) { this.url = url; this.options = options; }
        };
        const worker = createCompilerWorker();
        assert.equal(worker.url.pathname.split("/").at(-1), "@wie-arm-worker");
        assert.deepEqual(worker.options, { type: "module" });
        globalThis.Worker = class {
            constructor() { throw new Error("worker blocked"); }
        };
        assert.throws(createCompilerWorker, /worker blocked/);
    } finally {
        globalThis.Worker = previous;
    }
});
