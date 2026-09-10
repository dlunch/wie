import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import http from "node:http";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import webpack from "webpack";
import commonConfig from "../../wie-web/webpack.config.common.ts";

const { chromium } = createRequire(import.meta.url)("playwright");
const root = path.resolve(import.meta.dirname, "../..");
const host = path.join(import.meta.dirname, "browser-host");
const compiler = path.join(root, "wie-arm-wasm/compiler");
const env = { ...process.env, PATH: `${path.join(os.homedir(), ".cargo/bin")}${path.delimiter}${process.env.PATH ?? ""}` };

for (const mode of ["development", "production"]) {
    test(`backend-owned browser compiler and lifetime (${mode})`, { timeout: 240_000 }, async () => {
        for (const crate of [compiler, host]) {
            const build = spawnSync("wasm-pack", ["build", crate, "--target", "bundler", mode === "development" ? "--dev" : "--release", "--", "--locked", "--offline"], { env, encoding: "utf8" });
            assert.equal(build.status, 0, build.stdout + build.stderr);
        }
        const output = path.join(host, "dist", mode);
        await new Promise((resolve, reject) => {
            const bundler = webpack({
                mode,
                entry: path.join(import.meta.dirname, "browser-entry.js"),
                output: { path: output, filename: "test.js", clean: true },
                resolve: commonConfig(mode).resolve,
            });
            bundler.run((error, stats) => bundler.close(closeError => {
                if (error || closeError || stats.hasErrors()) reject(error || closeError || new Error(stats.toString()));
                else resolve();
            }));
        });
        let rejectCompiler = null;
        let compiler404s = 0;
        const server = http.createServer(async (request, response) => {
            try {
                const pathname = new URL(request.url, "http://localhost").pathname;
                assert(pathname.startsWith("/nested/backend/"));
                response.setHeader("Cache-Control", "no-store");
                if (pathname.endsWith("/")) {
                    response.setHeader("Content-Type", "text/html");
                    response.end('<script src="test.js"></script>');
                } else {
                    response.setHeader("Content-Type", pathname.endsWith(".wasm") ? "application/wasm" : "text/javascript");
                    const bytes = await readFile(path.join(output, pathname.slice("/nested/backend/".length)));
                    if (rejectCompiler && pathname.endsWith(".wasm")
                        && WebAssembly.Module.exports(new WebAssembly.Module(bytes)).some(entry => entry.name === "compile_request")) {
                        compiler404s++;
                        await rejectCompiler();
                        response.writeHead(404).end();
                    } else {
                        response.end(bytes);
                    }
                }
            } catch {
                response.writeHead(404).end();
            }
        });
        await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
        let browser;
        try {
            browser = await chromium.launch({ headless: true });
            const page = await browser.newPage();
            const errors = [];
            page.on("pageerror", error => errors.push(String(error)));
            await page.addInitScript(() => {
                globalThis.workers = [];
                const NativeWorker = Worker;
                globalThis.Worker = class extends NativeWorker {
                    constructor(...args) {
                        if (globalThis.failWorker) throw Error("test worker creation failure");
                        super(...args);
                        this.messages = [];
                        this.received = [];
                        this.errors = [];
                        this.terminated = false;
                        this.addEventListener("message", event => {
                            this.mutateResponse?.(event.data);
                            this.received.push(event.data);
                        });
                        this.addEventListener("error", event => this.errors.push(event.message));
                        workers.push(this);
                    }
                    postMessage(message, ...args) {
                        const { payload } = message;
                        const memory = globalThis.backendTest?.wasm.memory.buffer;
                        const before = memory && new Uint8Array(memory, 0, 64).slice();
                        const record = {
                            request: message.request,
                            payload: payload instanceof Uint8Array ? new TextDecoder().decode(payload) : payload,
                            bytePayload: payload instanceof Uint8Array,
                            buffer: payload.buffer,
                            transferred: args[0]?.length === 1 && args[0][0] === payload.buffer,
                            separate: payload.buffer !== memory,
                        };
                        this.messages.push(record);
                        if (this.failPost) throw Error("test postMessage failure");
                        super.postMessage(message, ...args);
                        record.detached = payload.buffer?.byteLength === 0;
                        record.memoryIntact = !memory || (memory.byteLength > 0 && memory === backendTest.wasm.memory.buffer
                            && before.every((byte, index) => byte === new Uint8Array(memory)[index]));
                    }
                    terminate() { this.terminated = true; super.terminate(); }
                };
                const instantiate = WebAssembly.instantiate;
                globalThis.installations = [];
                WebAssembly.instantiate = (module, imports) => {
                    if (!imports?.wie) return instantiate(module, imports);
                    installations.push(imports.wie);
                    if (globalThis.delayInstallation) {
                        return new Promise((resolve, reject) => {
                            globalThis.finishInstallation = () => instantiate(module, imports).then(resolve, reject);
                            globalThis.rejectInstallation = () => reject(Error("test late installation rejection"));
                        });
                    }
                    if (globalThis.trapAfterStore || globalThis.invalidExit) {
                        // region_0(frame, access): store(access, 0, 4, 1); trap or return an invalid exit.
                        const exit = globalThis.invalidExit ?? { type: 127, instructions: [0] };
                        module = new WebAssembly.Module(Uint8Array.from([
                            0,97,115,109,1,0,0,0,1,15,2,96,4,127,127,127,127,1,127,96,2,127,127,1,exit.type,
                            2,13,1,3,119,105,101,5,115,116,111,114,101,0,0,3,2,1,1,
                            7,12,1,8,114,101,103,105,111,110,95,48,0,1,
                            10,15 + exit.instructions.length,1,13 + exit.instructions.length,
                            0,32,1,65,0,65,4,65,1,16,0,26,...exit.instructions,11,
                        ]));
                    }
                    return instantiate(module, imports);
                };
            });
            await page.goto(`http://127.0.0.1:${server.address().port}/nested/backend/`);
            await page.waitForFunction(() => globalThis.backendTest);
            const result = await page.evaluate(async () => {
                const check = (condition, message) => { if (!condition) throw Error(message); };
                const sleep = () => new Promise(resolve => setTimeout(resolve, 5));
                const until = async predicate => {
                    for (let attempt = 0; attempt < 2000; attempt++) {
                        const result = predicate();
                        if (result) return result;
                        await sleep();
                    }
                    throw Error("backend test timed out");
                };
                const request = (id, regions = 1, base = 4096) => JSON.stringify({
                    session: 7, request: id,
                    regions: Array.from({ length: regions }, (_, index) => ({
                        ir: {
                            entry: { pc: base + index * 4, thumb: true, cpu_mode: 16 },
                            blocks: [{ instructions: [
                                { pc: base + index * 4, size: 2, condition: "Always", operation: { Alu: {
                                    op: "Add", destination: 0, left: { Register: 0 },
                                    right: { value: { Immediate: 1 }, shift: "Lsl", amount: { Immediate: 0 } }, set_flags: false,
                                } } },
                                { pc: base + index * 4 + 2, size: 2, condition: "Always", operation: { Branch: { target: { Immediate: base + index * 4 }, link: null, exchange: false } } },
                            ] }],
                        }, source: [{ page: 0, version: 1 }], expected_old: null,
                    })),
                });
                const { Probe, wasm } = backendTest;
                check(workers.length === 0, "loading the host created a worker");
                failWorker = true;
                const unavailable = new Probe(7n);
                for (let id = 1; id <= 2; id++) {
                    let failed = false;
                    try { unavailable.submit(request(id)); } catch (error) { failed = String(error).includes("worker creation"); }
                    check(failed, "worker creation failure was not reported through admission");
                }
                check(unavailable.poll() === "" && workers.length === 0 && installations.length === 0,
                    "unavailable compiler queued work or loaded a fallback module");
                unavailable.shutdown();
                unavailable.free();
                failWorker = false;
                const probe = new Probe(7n);
                check(workers.length === 1, "backend did not create exactly one worker");
                check(probe.submit(request(1)) === 0, "first compile request rejected");
                const first = JSON.parse(await until(() => probe.poll()));
                check(first.request === 1 && first.regions === 1, JSON.stringify(first));
                const sent = workers[0].messages[0];
                check(sent.bytePayload && sent.transferred && sent.separate && sent.detached && sent.memoryIntact,
                    "compile payload was not transferred independently of host memory");
                check(JSON.stringify(JSON.parse(sent.payload)) === request(1), "transport changed the request JSON");
                check(installations[0].load === wasm.wie_jit_load && installations[0].store === wasm.wie_jit_store
                    && installations[0].sample_prepare === wasm.wie_jit_sample_prepare
                    && installations[0].word_range === wasm.wie_jit_word_range && installations[0].memory === wasm.memory,
                "generated imports do not reference raw host exports");
                const budget = JSON.parse(probe.execute(1n, 0xfffffff0, 10, 100));
                check(budget.exit === 2 && budget.executed === 10 && budget.r0 === 5 && budget.budget === 0, JSON.stringify(budget));
                const end = JSON.parse(probe.execute(1n, 4098, 100, 100));
                check(end.exit === 3 && end.executed === 1 && end.r0 === 1, JSON.stringify(end));
                const sampled = JSON.parse(probe.execute(1n, 0xfffffff0, 100, 1));
                check(sampled.exit === 1 && sampled.executed === 1 && sampled.entry === 32768
                    && JSON.stringify(sampled.samples) === "[[4096,48,77,32768]]", JSON.stringify(sampled));
                const branchSample = JSON.parse(probe.execute(1n, 0xfffffff0, 100, 2));
                check(branchSample.exit === 1 && branchSample.executed === 2 && branchSample.entry === 4096
                    && JSON.stringify(branchSample.samples) === "[[4098,48,77,32768]]", JSON.stringify(branchSample));
                const targetSample = JSON.parse(probe.execute(1n, 0xfffffff0, 100, 3));
                check(targetSample.exit === 1 && targetSample.executed === 3 && targetSample.entry === 4096
                    && JSON.stringify(targetSample.samples) === "[[4096,48,77,4096]]", JSON.stringify(targetSample));
                const rejectedManifests = [];
                for (const [field, value] of Object.entries({
                    entry: { pc: 4098, thumb: true, cpu_mode: 16 },
                    source: [{ page: 0, version: 2 }],
                    export: "region_1",
                    expected_old: { slot: 1, generation: 1 },
                })) {
                    const replacement = JSON.parse(request(91 + rejectedManifests.length));
                    replacement.regions[0].expected_old = { slot: 0, generation: 1 };
                    const expected = { entry: replacement.regions[0].ir.entry, source: replacement.regions[0].source,
                        export: "region_0", expected_old: replacement.regions[0].expected_old };
                    const installedBefore = installations.length;
                    workers[0].mutateResponse = data => {
                        const manifest = JSON.parse(data.manifest);
                        check(data.request === String(replacement.request) && data.module instanceof WebAssembly.Module
                            && JSON.stringify(manifest) === JSON.stringify([expected]), "compiler response fixture was already invalid");
                        manifest[0][field] = value;
                        data.manifest = JSON.stringify(manifest);
                    };
                    check(probe.submit(JSON.stringify(replacement)) === 0, `manifest ${field} request rejected before compilation`);
                    const rejected = JSON.parse(await until(() => probe.poll()));
                    workers[0].mutateResponse = null;
                    check(rejected.request === replacement.request && rejected.error === "compiler manifest does not match its request",
                        `manifest ${field} corruption was not rejected: ${JSON.stringify(rejected)}`);
                    check(installations.length === installedBefore && probe.poll() === "", `manifest ${field} corruption instantiated or completed twice`);
                    check(JSON.parse(probe.execute(1n, 0xfffffff0, 4, 100)).r0 === 2, `manifest ${field} corruption damaged the old handle`);
                    rejectedManifests.push(field);
                }
                probe.retire(1n);
                for (let id = 2; id <= 10; id++) {
                    check(probe.submit(request(id)) === 0, "retirement did not return capacity");
                    const completion = JSON.parse(await until(() => probe.poll()));
                    check(completion.request === id && completion.regions === 1, JSON.stringify(completion));
                    probe.retire(BigInt(id));
                }
                const invalid = JSON.parse(request(11));
                invalid.regions[0].ir.entry.cpu_mode = 0;
                check(probe.submit(JSON.stringify(invalid)) === 0, "invalid IR was not admitted to compiler");
                const rejected = JSON.parse(await until(() => probe.poll()));
                check(rejected.request === 11 && typeof rejected.error === "string", "compile rejection lost request ID");
                check(probe.poll() === "", "compiler failure returned a duplicate completion");
                const worker = workers[0];
                const malformed = new TextEncoder().encode('{"session":');
                const installedBefore = installations.length;
                worker.postMessage({ request: "malformed-json", payload: malformed }, [malformed.buffer]);
                const malformedResponse = await until(() => worker.received.find(value => value.request === "malformed-json"));
                check(typeof malformedResponse.error === "string" && malformedResponse.error.includes("EOF"),
                    "malformed JSON bytes did not return a parse error with the request ID");
                worker.postMessage({ request: "old-string", payload: request(90) });
                const stringResponse = await until(() => worker.received.find(value => value.request === "old-string"));
                check(typeof stringResponse.error === "string" && stringResponse.error.includes("bytes"),
                    "worker accepted the obsolete string payload");
                check(probe.poll() === "" && installations.length === installedBefore, "invalid input installed a module");
                worker.failPost = true;
                check(probe.submit(request(90)) === 0, "post failure unexpectedly changed admission");
                const postFailure = JSON.parse(await until(() => probe.poll()));
                check(postFailure.request === 90 && postFailure.error.includes("test postMessage failure"),
                    "post failure lost its request or error");
                for (let attempt = 0; attempt < 3; attempt++) check(probe.poll() === "", "post failure completed more than once");
                check(!worker.received.some(value => value.request === "90"), "failed post reached the worker");
                worker.failPost = false;
                const liveGenerations = [];
                for (let id = 12; id <= 27; id++) {
                    check(probe.submit(request(id, 1, 4096 + (id - 12) * 4)) === 0, "live module capacity filled early");
                    check(JSON.parse(await until(() => probe.poll())).regions === 1, "live module failed to install");
                    liveGenerations.push(id);
                }
                check(probe.submit(request(28)) === 1, "live module limit exceeded sixteen");
                const countLiveBytes = liveGenerations.reduce((sum, id) => sum + worker.received.find(value => value.request === String(id)).encodedSize, 0);
                check(countLiveBytes + 512 * 1024 < 2 * 1024 * 1024 && countLiveBytes + 5 * 512 * 1024 < 4 * 1024 * 1024,
                    "peak module-count fixture also reaches a byte limit");
                const countReplacements = liveGenerations.slice(0, 5).map((generation, index) => {
                    const replacement = JSON.parse(request(100 + index, 1, 4096 + index * 4));
                    replacement.regions[0].expected_old = { slot: 0, generation };
                    return JSON.stringify(replacement);
                });
                const sentBeforePeak = worker.messages.length;
                delayInstallation = true;
                finishInstallation = null;
                check(probe.submit(countReplacements[0]) === 0, "first coexistence request rejected");
                await until(() => { probe.poll(); return globalThis.finishInstallation; });
                for (let index = 1; index < 4; index++) check(probe.submit(countReplacements[index]) === 0, "twenty coexisting modules were not admitted");
                check(probe.submit(countReplacements[4]) === 1, "peak module count admitted twenty-one coexisting modules");
                check(probe.poll() === "" && worker.messages.length === sentBeforePeak + 1
                    && !worker.received.some(value => value.request === "104"), "Busy peak-count request was submitted or completed");
                check(JSON.parse(probe.execute(12n, 0xfffffff0, 4, 100)).r0 === 2, "peak-count pressure retired the old handle");
                // Returning one unrelated module changes neither queue occupancy nor pending IR.
                probe.retire(BigInt(liveGenerations.pop()));
                check(probe.submit(countReplacements[4]) === 0, "retirement did not return peak module capacity");
                delayInstallation = false;
                finishInstallation();
                for (let index = 0; index < countReplacements.length; index++) {
                    const completion = JSON.parse(await until(() => probe.poll()));
                    check(completion.request === 100 + index && completion.regions === 1, JSON.stringify(completion));
                    if (index === 0) check(JSON.parse(probe.execute(12n, 0xfffffff0, 4, 100)).r0 === 2, "completion retired the old handle before its owner");
                    probe.retire(BigInt(liveGenerations[index]));
                    liveGenerations[index] = completion.request;
                }
                finishInstallation = null;
                check(probe.poll() === "" && worker.messages.length === sentBeforePeak + 5, "peak-count retries duplicated work or completion");
                check(probe.submit(request(105, 1, 4096 + liveGenerations.length * 4)) === 0, "could not refill the retired live module");
                check(JSON.parse(await until(() => probe.poll())).request === 105, "refilled live module did not install");
                liveGenerations.push(105);
                const merge = JSON.parse(request(28, 16));
                merge.regions.forEach((region, index) => { region.expected_old = { slot: 0, generation: liveGenerations[index] }; });
                check(probe.submit(JSON.stringify(merge)) === 0, "batch replacement could not reserve coexistence capacity");
                check(JSON.parse(await until(() => probe.poll())).regions === 16, "batch replacement did not install all exports");
                for (const id of liveGenerations) probe.retire(BigInt(id));
                check(JSON.parse(probe.execute(28n, 0xfffffff0, 4, 100)).r0 === 2, "retirement dropped the replacement instance");
                probe.retire(28n);
                const transfer = JSON.parse(request(29));
                const operations = [
                    { Alu: { op: "Move", destination: 0, left: { Immediate: 0 },
                        right: { value: { Immediate: 37 }, shift: "Lsl", amount: { Immediate: 0 } }, set_flags: false } },
                    { MultipleTransfer: { base: 1, registers: 129, increment: true, before: false, write_back: false, load: false } },
                    { Alu: { op: "Move", destination: 0, left: { Immediate: 0 },
                        right: { value: { Immediate: 0 }, shift: "Lsl", amount: { Immediate: 0 } }, set_flags: false } },
                    { Alu: { op: "Move", destination: 7, left: { Immediate: 0 },
                        right: { value: { Immediate: 0 }, shift: "Lsl", amount: { Immediate: 0 } }, set_flags: false } },
                    { MultipleTransfer: { base: 1, registers: 129, increment: true, before: false, write_back: false, load: true } },
                ];
                transfer.regions[0].ir.blocks[0].instructions = operations.map((operation, index) => ({
                    pc: 4096 + index * 2, size: 2, condition: "Always", operation,
                }));
                check(probe.submit(JSON.stringify(transfer)) === 0, "multiple-transfer request rejected");
                check(JSON.parse(await until(() => probe.poll())).regions === 1, "multiple-transfer request failed to install");
                probe.set_range_lengths(4, 4);
                const transferred = JSON.parse(probe.execute(29n, 4106, 100, 100));
                check(transferred.exit === 3 && transferred.executed === 5 && transferred.r0 === 37 && transferred.r7 === 77
                    && probe.stores === 0 && String(probe.memory()) === String(Uint8Array.from({ length: 256 }, (_, index) =>
                        index === 0 ? 37 : index === 4 ? 77 : 0)), JSON.stringify(transferred));
                probe.retire(29n);
                probe.shutdown();
                check(workers[0].terminated && workers[0].onmessage == null && workers[0].onerror == null, "shutdown retained worker listeners");
                probe.free();

                delayInstallation = true;
                const pending = new Probe(7n);
                check(pending.submit(request(1)) === 0, "pending request rejected");
                await until(() => { pending.poll(); return globalThis.finishInstallation; });
                for (let id = 2; id <= 5; id++) check(pending.submit(request(id)) === 0, "queue filled too early");
                check(pending.submit(request(6)) === 1, "queue admitted more than four waiting requests");
                check(workers[1].messages.length === 1, "multiple compile requests are in flight");
                pending.shutdown();
                check(pending.poll() === "" && workers[1].terminated && workers[1].onmessageerror == null, "pending shutdown retained state");
                pending.free();
                finishInstallation();
                await sleep();
                await sleep();

                finishInstallation = null;
                const lateRejected = new Probe(7n);
                check(lateRejected.submit(request(1)) === 0, "late rejection request rejected");
                await until(() => { lateRejected.poll(); return globalThis.finishInstallation; });
                lateRejected.shutdown();
                lateRejected.free();
                rejectInstallation();
                await sleep();
                await sleep();

                delayInstallation = false;
                const compiling = new Probe(7n);
                check(compiling.submit(request(1)) === 0, "compile shutdown request rejected");
                await until(() => { compiling.poll(); return workers[3].messages.length === 1; });
                compiling.shutdown();
                compiling.free();
                check(workers[3].terminated && workers[3].onmessage == null, "compile shutdown retained its worker");
                await sleep();

                trapAfterStore = true;
                const trapped = new Probe(7n);
                check(trapped.submit(request(1)) === 0, "trap test request rejected");
                check(JSON.parse(await until(() => trapped.poll())).regions === 1, "trap test failed to install");
                let trap = "";
                try { trapped.execute(1n, 0xfffffff0, 100, 100); } catch (error) { trap = String(error); }
                check(trap.includes("generated code trapped") && trapped.stores === 1, "generated trap replayed or lost its store");
                check(workers[4].terminated && workers[4].onmessage == null, "generated trap did not close the executor");
                try { trapped.execute(1n, 0xfffffff0, 100, 100); } catch { /* A closed executor cannot replay the store. */ }
                check(trapped.stores === 1, "closed executor ran generated code again");
                let closed = false;
                try { trapped.submit(request(2)); } catch (error) { closed = String(error).includes("closed"); }
                check(closed, "generated trap allowed later submissions");
                trapped.free();

                trapAfterStore = false;
                delayInstallation = true;
                finishInstallation = null;
                const irBudget = new Probe(7n);
                const large = JSON.parse(request(1));
                large.regions[0].source = Array.from({ length: 4000 }, (_, page) => ({ page, version: 1 }));
                check(irBudget.submit(JSON.stringify(large)) === 0, "single request below the IR limit was rejected");
                large.request = 2;
                check(irBudget.submit(JSON.stringify(large)) === 1, "queued request released its IR reservation");
                await until(() => { irBudget.poll(); return workers[5].messages.length === 1; });
                check(workers[5].messages[0].detached, "pending request payload was not detached");
                check(irBudget.submit(JSON.stringify(large)) === 1, "compiling request released its IR reservation");
                await until(() => { irBudget.poll(); return globalThis.finishInstallation; });
                check(irBudget.submit(JSON.stringify(large)) === 1, "concurrent request exceeded the total pending IR budget");
                await finishInstallation();
                await sleep();
                check(irBudget.submit(JSON.stringify(large)) === 1, "unconsumed completion released its IR reservation");
                check(JSON.parse(irBudget.poll()).request === 1, "held installation did not complete");
                check(irBudget.submit(JSON.stringify(large)) === 0, "consumed completion did not release its IR reservation");
                large.request = 3;
                large.regions[0].source = Array.from({ length: 9000 }, (_, page) => ({ page, version: 1 }));
                let oversized = false;
                try { irBudget.submit(JSON.stringify(large)); } catch (error) { oversized = String(error).includes("exceeds the IR budget"); }
                check(oversized, "an individually oversized request was not a permanent admission failure");
                irBudget.shutdown();
                irBudget.free();
                await sleep();
                await sleep();
                delayInstallation = false;
                for (const value of [5, 0.5, NaN, null, 0n]) {
                    const bytes = new Uint8Array(8);
                    new DataView(bytes.buffer).setFloat64(0, typeof value === "number" ? value : 0, true);
                    globalThis.invalidExit = value === null ? { type: 111, instructions: [208, 111] }
                        : typeof value === "bigint" ? { type: 126, instructions: [66, 0] }
                        : { type: 124, instructions: [68, ...bytes] };
                    const invalidExitProbe = new Probe(7n);
                    const worker = workers.at(-1);
                    try {
                        check(invalidExitProbe.submit(request(1)) === 0, "invalid-exit request rejected");
                        check(JSON.parse(await until(() => invalidExitProbe.poll())).regions === 1, "invalid-exit module did not install");
                        let failure = "";
                        try { invalidExitProbe.execute(1n, 0xfffffff0, 100, 100); } catch (error) { failure = String(error); }
                        check(failure.includes("compiled ABI returned invalid exit") && invalidExitProbe.stores === 1,
                            `invalid exit ${value} was coerced, or its store was replayed`);
                        check(worker.terminated && worker.onmessage == null, "invalid exit did not close the executor");
                        try { invalidExitProbe.execute(1n, 0xfffffff0, 100, 100); } catch { /* Already closed. */ }
                        check(invalidExitProbe.stores === 1, "closed executor replayed an invalid-exit store");
                    } finally {
                        globalThis.invalidExit = null;
                        invalidExitProbe.free();
                    }
                }
                return { budget, end, sampled, rejectedManifests, workerCount: workers.length,
                    peakCount: { liveBytes: countLiveBytes, fifthReservationBytes: countLiveBytes + 5 * 512 * 1024, rejected: "Busy", recovered: "Accepted" },
                    irBudget: { single: "Accepted", concurrent: "Busy", oversized: "Failed" } };
            });
            assert.equal(result.workerCount, 11);
            assert.deepEqual(errors, []);
            console.log(JSON.stringify({ mode, ...result }));

            const ranges = await page.evaluate(async () => {
                delayInstallation = false;
                const probe = new backendTest.Probe(7n);
                const worker = workers.at(-1);
                try {
                    const request = {
                        session: 7, request: 1,
                        regions: [{
                            ir: {
                                entry: { pc: 4096, thumb: true, cpu_mode: 16 },
                                blocks: [{ instructions: [{ pc: 4096, size: 2, condition: "Always", operation: { MultipleTransfer: {
                                    base: 0, registers: 6, increment: true, before: false, write_back: true, load: true,
                                } } }] }],
                            }, source: [{ page: 0, version: 1 }], expected_old: null,
                        }],
                    };
                    if (probe.submit(JSON.stringify(request)) !== 0) throw Error("range probe admission failed");
                    let completion = "";
                    for (let attempt = 0; attempt < 2000 && !completion; attempt++) {
                        completion = probe.poll();
                        if (!completion) await new Promise(resolve => setTimeout(resolve, 5));
                    }
                    if (!completion || JSON.parse(completion).regions !== 1) throw Error("range probe compilation failed: " + completion);
                    const results = [];
                    const wrapping = structuredClone(request);
                    wrapping.request = 2;
                    wrapping.regions[0].ir.blocks[0].instructions[0].pc = 4098;
                    wrapping.regions[0].ir.blocks[0].instructions.unshift({ pc: 4096, size: 2, condition: "Always", operation: {
                        Alu: { op: "Move", destination: 0, left: { Immediate: 0 },
                            right: { value: { Immediate: 0xfffffffc }, shift: "Lsl", amount: { Immediate: 0 } }, set_flags: false },
                    } });
                    if (probe.submit(JSON.stringify(wrapping)) !== 0) throw Error("wrapping range admission failed");
                    completion = "";
                    for (let attempt = 0; attempt < 2000 && !completion; attempt++) {
                        completion = probe.poll();
                        if (!completion) await new Promise(resolve => setTimeout(resolve, 5));
                    }
                    if (!completion || JSON.parse(completion).regions !== 1) throw Error("wrapping range compilation failed: " + completion);
                    const wrapped = JSON.parse(probe.execute(2n, 0xfffffff0, 2, 100));
                    if (wrapped.exit !== 4 || wrapped.executed !== 1 || wrapped.r0 !== 0xfffffffc || wrapped.pc !== 4098) {
                        throw Error("unmapped wrapping range did not decline: " + JSON.stringify(wrapped));
                    }
                    probe.retire(2n);
                    for (const [first, second, admitted] of [[0,8,false], [3,5,false], [4,0,false], [4,8,false], [8,0,true], [4,4,true]]) {
                        probe.set_range_lengths(first, second);
                        const memory = probe.memory();
                        const result = JSON.parse(probe.execute(1n, 0xfffffff0, 1, 100));
                        if (result.exit !== (admitted ? 2 : 4) || result.executed !== Number(admitted)
                            || result.r0 !== (admitted ? 8 : 0) || result.pc !== (admitted ? 4098 : 4096)
                            || result.scratch !== (admitted ? first : 0) || String(probe.memory()) !== String(memory)) {
                            throw Error("unsafe or incorrect borrowed range: " + JSON.stringify({ first, second, result }));
                        }
                        results.push({ first, second, result });
                    }
                    return results;
                } finally {
                    probe.shutdown();
                    probe.free();
                    if (!worker.terminated || worker.onmessage != null) throw Error("range probe retained its worker");
                }
            });
            console.log(JSON.stringify({ mode, ranges }));

            const capacity = await page.evaluate(async () => {
                const check = (condition, message) => { if (!condition) throw Error(message); };
                const until = async predicate => {
                    for (let attempt = 0; attempt < 2000; attempt++) {
                        const value = predicate();
                        if (value) return value;
                        await new Promise(resolve => setTimeout(resolve, 5));
                    }
                    throw Error("capacity test timed out");
                };
                const reservation = 512 * 1024;
                const liveLimit = 2 * 1024 * 1024;
                const peakLimit = 4 * 1024 * 1024;
                const request = (id, base, dense = true, armRegions = 3) => ({
                    session: 7, request: id,
                    regions: [{
                        ir: {
                            entry: { pc: base, thumb: true, cpu_mode: 16 },
                            blocks: [{ instructions: [
                                { pc: base, size: 2, condition: "Always", operation: { Alu: {
                                    op: "Add", destination: 0, left: { Register: 0 },
                                    right: { value: { Immediate: 1 }, shift: "Lsl", amount: { Immediate: 0 } }, set_flags: true,
                                } } },
                                { pc: base + 2, size: 2, condition: "Always", operation: { Branch: {
                                    target: { Immediate: base }, link: null, exchange: false,
                                } } },
                            ] }],
                        }, source: [{ page: base & 0xffff0000, version: 1 }], expected_old: null,
                    }, ...Array.from({ length: armRegions }, (_, region) => {
                        const pc = base + (region + 1) * 1024;
                        return {
                            ir: {
                                entry: { pc, thumb: false, cpu_mode: 16 },
                                blocks: [{ instructions: Array.from({ length: dense ? 200 : 1 }, (_, index) => ({
                                    pc: pc + index * 4, size: 4, condition: "Le",
                                    operation: { MultipleTransfer: {
                                        base: 0, registers: 255, increment: true, before: false, write_back: false, load: true,
                                    } },
                                })) }],
                            }, source: [{ page: pc & 0xffff0000, version: 1 }], expected_old: null,
                        };
                    })],
                });
                delayInstallation = false;
                const probe = new backendTest.Probe(7n);
                const worker = workers.at(-1);
                try {
                    let oversized = "";
                    try { probe.submit(JSON.stringify(request(100, 4096, true, 8))); } catch (error) { oversized = String(error); }
                    check(oversized === "compile request exceeds the IR budget", `oversized admission: ${oversized}`);
                    for (let index = 0; index < 4; index++) check(probe.poll() === "", "failed admission produced a completion");
                    check(worker.messages.length === 0, "failed admission reached the worker");

                    const modules = new Map();
                    for (let id = 1; id <= 7; id++) {
                        check(probe.submit(JSON.stringify(request(id, 4096 + (id - 1) * 16384))) === 0,
                            `dense request ${id} rejected with live sizes ${JSON.stringify([...modules.values()])}`);
                        const completion = JSON.parse(await until(() => probe.poll()));
                        check(completion.request === id && completion.regions === 4, JSON.stringify(completion));
                        const bytes = worker.received.find(value => value.request === String(id)).encodedSize;
                        check(bytes > 0 && bytes <= reservation, `invalid actual output size: ${bytes}`);
                        modules.set(id, bytes);
                    }
                    const liveBytes = [...modules.values()].reduce((sum, bytes) => sum + bytes, 0);
                    check(modules.size + 1 <= 16 && liveBytes + reservation > liveLimit && liveBytes <= liveLimit,
                        `fixture does not isolate live byte pressure: ${liveBytes}`);
                    const retry = JSON.stringify(request(8, 4096 + 7 * 16384));
                    check(probe.submit(retry) === 1, "live encoded bytes did not reject the eighth module");
                    check(probe.poll() === "" && worker.messages.length === 7, "Busy admission queued work");
                    probe.retire(7n);
                    modules.delete(7);
                    check(probe.submit(retry) === 0, "retirement did not recover live byte capacity");
                    const recovered = JSON.parse(await until(() => probe.poll()));
                    check(recovered.request === 8 && recovered.regions === 4, JSON.stringify(recovered));
                    modules.set(8, worker.received.find(value => value.request === "8").encodedSize);

                    // Keep one real installation pending; the remaining reservations contain only short replacement IR.
                    delayInstallation = true;
                    finishInstallation = null;
                    const replacements = [];
                    for (let old = 1; old <= 5; old++) {
                        const replacement = request(old + 8, 4096 + (old - 1) * 16384, false);
                        replacement.regions.forEach((region, slot) => { region.expected_old = { slot, generation: old }; });
                        replacements.push(JSON.stringify(replacement));
                    }
                    const partial = JSON.parse(replacements[0]);
                    partial.regions.pop();
                    check(modules.size + 1 <= 16 && modules.size + 1 <= 20 && liveBytes + reservation <= peakLimit
                        && liveBytes + reservation > liveLimit && liveBytes - modules.get(1) + reservation <= liveLimit,
                        "partial replacement fixture does not isolate the old module's live bytes");
                    check(probe.submit(JSON.stringify(partial)) === 1, "partial export replacement reclaimed the old module's live bytes");
                    check(probe.poll() === "" && worker.messages.length === 8, "Busy partial replacement queued compilation");
                    check(JSON.parse(probe.execute(1n, 0xfffffff0, 4, 100)).r0 === 2, "partial replacement damaged the old handle");
                    check(probe.submit(replacements[0]) === 0, "replacement could not reserve coexistence bytes");
                    await until(() => { probe.poll(); return globalThis.finishInstallation; });
                    check(JSON.parse(probe.execute(1n, 0xfffffff0, 4, 100)).r0 === 2, "pending full replacement retired the old handle");
                    for (let index = 1; index < 4; index++) check(probe.submit(replacements[index]) === 0, "coexistence capacity filled early");
                    const coexistBytes = [...modules.values()].reduce((sum, bytes) => sum + bytes, 0);
                    const rejectedPeak = coexistBytes + 5 * reservation;
                    check(modules.size + 5 <= 20 && coexistBytes + 4 * reservation <= peakLimit && rejectedPeak > peakLimit,
                        `fixture does not isolate peak byte pressure: ${rejectedPeak}`);
                    check(coexistBytes - modules.get(5) + reservation <= liveLimit, "replacement would exceed the live byte limit");
                    check(probe.submit(replacements[4]) === 1, "fifth reservation exceeded coexistence bytes");
                    check(probe.poll() === "" && worker.messages.length === 9, "Busy reservation reached the worker");
                    const afterRetirePeak = coexistBytes - modules.get(8) + 5 * reservation;
                    check(afterRetirePeak <= peakLimit, "retirement fixture cannot recover peak capacity");
                    probe.retire(8n);
                    modules.delete(8);
                    check(probe.submit(replacements[4]) === 0, "retirement did not recover coexistence reservations");
                    delayInstallation = false;
                    finishInstallation();
                    const completions = [];
                    for (let id = 9; id <= 13; id++) {
                        const completion = JSON.parse(await until(() => probe.poll()));
                        check(completion.request === id && completion.regions === 4, JSON.stringify(completion));
                        completions.push(completion.request);
                        if (id === 9) {
                            check(JSON.parse(probe.execute(1n, 0xfffffff0, 4, 100)).r0 === 2, "full replacement retired the old handle before its owner");
                        }
                        probe.retire(BigInt(id - 8));
                    }
                    for (let index = 0; index < 4; index++) check(probe.poll() === "", "reservation returned a duplicate completion");
                    check(worker.messages.length === 13 && worker.received.filter(value => value.request).length === 13,
                        "admission failure or Busy retry compiled more than once");
                    const execution = JSON.parse(probe.execute(9n, 0xfffffff0, 4, 100));
                    check(execution.exit === 2 && execution.executed === 4 && execution.r0 === 2, JSON.stringify(execution));
                    check(worker.errors.length === 0, JSON.stringify(worker.errors));
                    return { oversized, liveBytes, rejectedLive: liveBytes + reservation, coexistBytes, rejectedPeak,
                        partialReplacement: "Busy", fullReplacement: "Accepted",
                        afterRetirePeak, completions, execution, encodedSizes: worker.received.filter(value => value.request).map(value => value.encodedSize) };
                } finally {
                    delayInstallation = false;
                    probe.shutdown();
                    probe.free();
                    check(worker.terminated && worker.onmessage == null && worker.onerror == null, "capacity test retained its worker");
                }
            });
            console.log(JSON.stringify({ mode, capacity }));

            rejectCompiler = () => {};
            const admissions = await page.evaluate(() => {
                globalThis.loadingProbe = new backendTest.Probe(7n);
                globalThis.loadingCompletions = [];
                const request = {
                    session: 7, request: 0,
                    regions: [{
                        ir: {
                            entry: { pc: 4096, thumb: true, cpu_mode: 16 },
                            blocks: [{ instructions: [{ pc: 4096, size: 2, condition: "Always",
                                operation: { Branch: { target: { Immediate: 4096 }, link: null, exchange: false } } }] }],
                        },
                        source: [{ page: 0, version: 1 }], expected_old: null,
                    }],
                };
                const admissions = [];
                for (let id = 1; id <= 4; id++) {
                    request.request = id;
                    admissions.push(loadingProbe.submit(JSON.stringify(request)));
                }
                globalThis.loadingRequest = request;
                return admissions;
            });
            assert.deepEqual(admissions, [0, 0, 0, 0]);
            try {
                await page.waitForFunction(() => {
                    const completion = loadingProbe.poll();
                    if (completion) loadingCompletions.push(JSON.parse(completion));
                    return loadingCompletions.length === 4;
                }, null, { timeout: 5000 });
            } finally {
                console.log(JSON.stringify({ mode, compiler404s, ...await page.evaluate(() => ({
                    loadingCompletions, sent: workers.at(-1).messages.length,
                    received: workers.at(-1).received, workerErrors: workers.at(-1).errors,
                })) }));
            }
            const loadingFailure = await page.evaluate(() => {
                const worker = workers.at(-1);
                const installationsBefore = installations.length;
                const completions = loadingCompletions.slice();
                const extraPolls = Array.from({ length: 4 }, () => loadingProbe.poll());
                loadingRequest.request = 5;
                let nextSubmission = "";
                try { loadingProbe.submit(JSON.stringify(loadingRequest)); } catch (error) { nextSubmission = String(error); }
                loadingProbe.shutdown();
                const afterShutdown = loadingProbe.poll();
                loadingProbe.free();
                worker.dispatchEvent(new MessageEvent("message", { data: { ready: true } }));
                return { completions, extraPolls, nextSubmission, afterShutdown,
                    sent: worker.messages.length, workerErrors: worker.errors,
                    closed: worker.terminated && worker.onmessage == null && worker.onerror == null && worker.onmessageerror == null,
                    lateInstalls: installations.length - installationsBefore };
            });
            assert.equal(compiler404s, 1);
            assert.deepEqual(loadingFailure.completions.map(completion => completion.request), [1, 2, 3, 4]);
            assert(loadingFailure.completions.every(completion => /compiler initialization/i.test(completion.error)));
            assert.deepEqual(loadingFailure.extraPolls, ["", "", "", ""]);
            assert.match(loadingFailure.nextSubmission, /compiler initialization/i);
            assert.equal(loadingFailure.afterShutdown, "");
            assert.equal(loadingFailure.sent, 0);
            assert.deepEqual(loadingFailure.workerErrors, []);
            assert(loadingFailure.closed);
            assert.equal(loadingFailure.lateInstalls, 0);

            let releaseResponse;
            let observeRequest;
            const responseGate = new Promise(resolve => { releaseResponse = resolve; });
            const requestObserved = new Promise(resolve => { observeRequest = resolve; });
            rejectCompiler = () => { observeRequest(); return responseGate; };
            const installationsBeforeClose = await page.evaluate(() => installations.length);
            try {
                assert.equal(await page.evaluate(() => {
                    globalThis.closedLoadingProbe = new backendTest.Probe(7n);
                    return closedLoadingProbe.submit(JSON.stringify(loadingRequest));
                }), 0);
                await requestObserved;
                await page.evaluate(() => closedLoadingProbe.shutdown());
                releaseResponse();
                const closedLoading = await page.evaluate(async () => {
                    await new Promise(resolve => setTimeout(resolve, 50));
                    const worker = workers.at(-1);
                    const completion = closedLoadingProbe.poll();
                    closedLoadingProbe.free();
                    return { completion, sent: worker.messages.length, received: worker.received.length,
                        closed: worker.terminated && worker.onmessage == null, installations: installations.length };
                });
                assert.equal(compiler404s, 2);
                assert.equal(closedLoading.completion, "");
                assert.equal(closedLoading.sent, 0);
                assert.equal(closedLoading.received, 0);
                assert(closedLoading.closed);
                assert.equal(closedLoading.installations, installationsBeforeClose);
            } finally {
                releaseResponse();
            }
            assert.deepEqual(errors, []);
        } finally {
            await browser?.close();
            await new Promise(resolve => server.close(resolve));
        }
    });
}
