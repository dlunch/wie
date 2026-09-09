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
                        this.addEventListener("message", event => this.received.push(event.data));
                        this.addEventListener("error", event => this.errors.push(event.message));
                        workers.push(this);
                    }
                    postMessage(message) { this.messages.push(message); super.postMessage(message); }
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
                    if (globalThis.trapAfterStore) {
                        // region_0(frame, access): store(access, 0, 4, 1); unreachable.
                        module = new WebAssembly.Module(Uint8Array.from([
                            0,97,115,109,1,0,0,0,1,15,2,96,4,127,127,127,127,1,127,96,2,127,127,1,127,
                            2,13,1,3,119,105,101,5,115,116,111,114,101,0,0,3,2,1,1,
                            7,12,1,8,114,101,103,105,111,110,95,48,0,1,
                            10,16,1,14,0,32,1,65,0,65,4,65,1,16,0,26,0,11,
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
                check(installations[0].load === wasm.wie_jit_load && installations[0].store === wasm.wie_jit_store
                    && installations[0].sample_prepare === wasm.wie_jit_sample_prepare
                    && installations[0].word_range === wasm.wie_jit_word_range && installations[0].memory === wasm.memory,
                "generated imports do not reference raw host exports");
                const budget = JSON.parse(probe.execute(1n, 0xfffffff0, 10, 100));
                check(budget.exit === 2 && budget.executed === 10 && budget.r0 === 5 && budget.budget === 0, JSON.stringify(budget));
                const end = JSON.parse(probe.execute(1n, 4098, 100, 100));
                check(end.exit === 3 && end.executed === 1 && end.r0 === 1, JSON.stringify(end));
                const sampled = JSON.parse(probe.execute(1n, 0xfffffff0, 100, 1));
                check(sampled.exit === 1 && sampled.executed === 1 && JSON.stringify(sampled.samples) === "[[4096,48,77]]", JSON.stringify(sampled));
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
                for (let id = 12; id <= 19; id++) {
                    check(probe.submit(request(id, 1, 4096 + (id - 12) * 4)) === 0, "live module capacity filled early");
                    check(JSON.parse(await until(() => probe.poll())).regions === 1, "live module failed to install");
                }
                check(probe.submit(request(20)) === 1, "live module limit exceeded eight");
                const merge = JSON.parse(request(20, 8));
                merge.regions.forEach((region, index) => { region.expected_old = { slot: 0, generation: index + 12 }; });
                check(probe.submit(JSON.stringify(merge)) === 0, "batch replacement could not reserve coexistence capacity");
                check(JSON.parse(await until(() => probe.poll())).regions === 8, "batch replacement did not install all exports");
                for (let id = 12; id <= 19; id++) probe.retire(BigInt(id));
                check(JSON.parse(probe.execute(20n, 0xfffffff0, 4, 100)).r0 === 2, "retirement dropped the replacement instance");
                probe.retire(20n);
                const transfer = JSON.parse(request(21));
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
                const transferred = JSON.parse(probe.execute(21n, 4106, 100, 100));
                check(transferred.exit === 3 && transferred.executed === 5 && transferred.r0 === 37 && transferred.r7 === 77
                    && probe.stores === 2, JSON.stringify(transferred));
                probe.retire(21n);
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
                await until(() => { irBudget.poll(); return globalThis.finishInstallation; });
                large.request = 2;
                check(irBudget.submit(JSON.stringify(large)) === 1, "concurrent request exceeded the total pending IR budget");
                large.request = 3;
                large.regions[0].source = Array.from({ length: 9000 }, (_, page) => ({ page, version: 1 }));
                let oversized = false;
                try { irBudget.submit(JSON.stringify(large)); } catch (error) { oversized = String(error).includes("exceeds the IR budget"); }
                check(oversized, "an individually oversized request was not a permanent admission failure");
                irBudget.shutdown();
                irBudget.free();
                finishInstallation();
                await sleep();
                await sleep();
                return { budget, end, sampled, workerCount: workers.length, irBudget: { single: "Accepted", concurrent: "Busy", oversized: "Failed" } };
            });
            assert.equal(result.workerCount, 6);
            assert.deepEqual(errors, []);
            console.log(JSON.stringify({ mode, ...result }));

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
                                blocks: [{ instructions: Array.from({ length: dense ? 256 : 1 }, (_, index) => ({
                                    pc: pc + index * 4, size: 4, condition: "Le",
                                    operation: { Alu: {
                                        op: "ReverseSubCarry", destination: 0, left: { Register: 0 },
                                        right: { value: { Register: 1 }, shift: "Lsl", amount: { Register: 2 } }, set_flags: true,
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
                    try { probe.submit(JSON.stringify(request(100, 4096, true, 4))); } catch (error) { oversized = String(error); }
                    check(oversized === "compile request exceeds the IR budget", `oversized admission: ${oversized}`);
                    for (let index = 0; index < 4; index++) check(probe.poll() === "", "failed admission produced a completion");
                    check(worker.messages.length === 0, "failed admission reached the worker");

                    const modules = new Map();
                    for (let id = 1; id <= 7; id++) {
                        check(probe.submit(JSON.stringify(request(id, 4096 + (id - 1) * 16384))) === 0, `dense request ${id} rejected`);
                        const completion = JSON.parse(await until(() => probe.poll()));
                        check(completion.request === id && completion.regions === 4, JSON.stringify(completion));
                        const bytes = worker.received.find(value => value.request === String(id)).encodedSize;
                        check(bytes > 0 && bytes <= reservation, `invalid actual output size: ${bytes}`);
                        modules.set(id, bytes);
                    }
                    const liveBytes = [...modules.values()].reduce((sum, bytes) => sum + bytes, 0);
                    check(modules.size + 1 <= 8 && liveBytes + reservation > liveLimit && liveBytes <= liveLimit,
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
                    check(probe.submit(replacements[0]) === 0, "replacement could not reserve coexistence bytes");
                    await until(() => { probe.poll(); return globalThis.finishInstallation; });
                    for (let index = 1; index < 4; index++) check(probe.submit(replacements[index]) === 0, "coexistence capacity filled early");
                    const coexistBytes = [...modules.values()].reduce((sum, bytes) => sum + bytes, 0);
                    const rejectedPeak = coexistBytes + 5 * reservation;
                    check(modules.size + 5 <= 12 && coexistBytes + 4 * reservation <= peakLimit && rejectedPeak > peakLimit,
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
                        probe.retire(BigInt(id - 8));
                    }
                    for (let index = 0; index < 4; index++) check(probe.poll() === "", "reservation returned a duplicate completion");
                    check(worker.messages.length === 13 && worker.received.filter(value => value.request).length === 13,
                        "admission failure or Busy retry compiled more than once");
                    const execution = JSON.parse(probe.execute(9n, 0xfffffff0, 4, 100));
                    check(execution.exit === 2 && execution.executed === 4 && execution.r0 === 2, JSON.stringify(execution));
                    check(worker.errors.length === 0, JSON.stringify(worker.errors));
                    return { oversized, liveBytes, rejectedLive: liveBytes + reservation, coexistBytes, rejectedPeak,
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
