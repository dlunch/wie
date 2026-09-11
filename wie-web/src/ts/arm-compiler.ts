declare const __webpack_hash__: string;

interface ArmArtifactBytes {
    bytes: Uint8Array;
    manifest: Uint8Array;
}

interface ArmCacheRecord extends ArmArtifactBytes {
    digest: string;
}

interface ArmCacheLookup {
    key?: string;
    artifact?: ArmCacheRecord;
}

let latestModule: { key: string; digest: string; module: WebAssembly.Module } | undefined;

async function duringPreparation<T>(
    deadline: number, signal: AbortSignal, work: (check: () => void, taskSignal: AbortSignal) => Promise<T>,
): Promise<T> {
    const controller = new AbortController();
    const check = () => {
        controller.signal.throwIfAborted();
        signal.throwIfAborted();
        if (performance.now() >= deadline) throw new Error("ARM AOT preparation timed out");
    };
    let interrupt!: (error: unknown) => void;
    const interrupted = new Promise<never>((_, reject) => { interrupt = reject; });
    const abort = () => {
        controller.abort(signal.reason);
        interrupt(signal.reason);
    };
    signal.addEventListener("abort", abort, { once: true });
    const timeout = setTimeout(() => {
        const failure = new Error("ARM AOT preparation timed out");
        controller.abort(failure);
        interrupt(failure);
    }, Math.max(0, deadline - performance.now()));
    try {
        check();
        const result = await Promise.race([interrupted, work(check, controller.signal)]);
        check();
        return result;
    } catch (error) {
        controller.abort(error);
        throw error;
    } finally {
        controller.abort();
        clearTimeout(timeout);
        signal.removeEventListener("abort", abort);
    }
}

async function sha256(bytes: Uint8Array, check: () => void): Promise<string> {
    check();
    const digest = await crypto.subtle.digest("SHA-256", bytes as Uint8Array<ArrayBuffer>);
    check();
    return Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, "0")).join("");
}

async function artifactDigest(artifact: ArmArtifactBytes, key: string, check: () => void): Promise<string> {
    const bytes = await sha256(artifact.bytes, check);
    const manifest = await sha256(artifact.manifest, check);
    return sha256(new TextEncoder().encode(JSON.stringify([key, bytes, manifest])), check);
}

async function accessArmCache(key: string, record: ArmCacheRecord | undefined, check: () => void, signal: AbortSignal): Promise<unknown> {
    let db: IDBDatabase | undefined;
    let transaction: IDBTransaction | undefined;
    const close = () => {
        try {
            transaction?.abort();
        } catch {
            // Commit may finish before the transaction's completion event arrives.
        }
        transaction = undefined;
        db?.close();
    };
    signal.addEventListener("abort", close, { once: true });
    try {
        return await new Promise((resolve, reject) => {
            check();
            const opening = indexedDB.open("wie_arm_aot");
            opening.onupgradeneeded = () => {
                try {
                    check();
                    opening.result.createObjectStore("artifacts");
                } catch (error) {
                    opening.transaction?.abort();
                    opening.result.close();
                    reject(error);
                }
            };
            opening.onerror = () => reject(opening.error);
            opening.onblocked = () => reject(new Error("ARM AOT cache blocked"));
            opening.onsuccess = () => {
                db = opening.result;
                try {
                    check();
                    const current = transaction = db.transaction("artifacts", record ? "readwrite" : "readonly");
                    const store = current.objectStore("artifacts");
                    const request = record ? store.put(record, key) : store.get(key);
                    current.oncomplete = () => {
                        transaction = undefined;
                        try {
                            check();
                            resolve(request.result);
                        } catch (error) {
                            reject(error);
                        }
                    };
                    current.onerror = current.onabort = () => {
                        transaction = undefined;
                        reject(current.error);
                    };
                } catch (error) {
                    // An open can finish after its deadline or a blocked notification.
                    db.close();
                    reject(error);
                }
            };
        });
    } finally {
        signal.removeEventListener("abort", close);
        close();
    }
}

export async function loadArmCache(input: Uint8Array, deadline: number, signal: AbortSignal): Promise<ArmCacheLookup> {
    const started = performance.now();
    const timing: Record<string, number> = {};
    let outcome = "miss";
    let key: string | undefined;
    try {
        return await duringPreparation(deadline, signal, async (check, taskSignal) => {
            try {
                key = `${__webpack_hash__}:${await sha256(input, check)}`;
                check();
                timing.keying = performance.now() - started;
                let phaseStarted = performance.now();
                const record = await accessArmCache(key, undefined, check, taskSignal);
                check();
                timing.read = performance.now() - phaseStarted;
                phaseStarted = performance.now();
                if (record === undefined) return { key };
                outcome = "corrupt";
                if (typeof record !== "object" || record === null ||
                    !("bytes" in record) || !(record.bytes instanceof Uint8Array) ||
                    !("manifest" in record) || !(record.manifest instanceof Uint8Array) ||
                    !("digest" in record) || typeof record.digest !== "string") return { key };
                const artifact = { bytes: record.bytes, manifest: record.manifest, digest: record.digest };
                const digest = await artifactDigest(artifact, key, check);
                check();
                timing.integrity = performance.now() - phaseStarted;
                if (digest !== artifact.digest) return { key };
                outcome = "persistent-hit";
                return { key, artifact };
            } catch {
                check();
                outcome = "unavailable";
                return { key };
            }
        });
    } catch (error) {
        outcome = String(error);
        throw error;
    } finally {
        console.info("ARM AOT cache lookup", { outcome, elapsedMs: performance.now() - started, phasesMs: timing });
    }
}

// This detached task owns only cache data, never an instance, imports or a warmup frame.
async function storeArmCache(record: ArmCacheRecord, key: string, deadline: number, signal: AbortSignal): Promise<void> {
    const started = performance.now();
    let outcome = "stored";
    try {
        await duringPreparation(deadline, signal, (check, taskSignal) => accessArmCache(key, record, check, taskSignal));
    } catch {
        outcome = signal.aborted || performance.now() >= deadline ? "skipped" : "failed";
    }
    console.info("ARM AOT cache store", { outcome, elapsedMs: performance.now() - started });
}

export function compilerTask(): Promise<void> {
    // Message tasks yield the main thread without the nested-timer clamp.
    return new Promise(resolve => {
        const { port1, port2 } = new MessageChannel();
        port1.onmessage = () => {
            port1.close();
            port2.close();
            resolve();
        };
        port2.postMessage(null);
    });
}

export async function compileArm(
    artifact: ArmArtifactBytes, key: string | undefined, cachedDigest: string | undefined, imports: WebAssembly.Imports,
    frame: number, regionCount: number, deadline: number, signal: AbortSignal,
): Promise<Function> {
    let stopped: unknown;
    let failed = false;
    const started = performance.now();
    const timing: Record<string, number> = {};
    let cache = cachedDigest === undefined ? "miss" : "persistent-hit";
    let memory: WebAssembly.Memory | undefined = imports.wie.memory as WebAssembly.Memory;
    const memoryBefore = memory.buffer.byteLength;
    try {
        return await duringPreparation(deadline, signal, async check => {
            check();
            let digest = cachedDigest;
            if (key !== undefined && digest === undefined) {
                try {
                    digest = await artifactDigest(artifact, key, check);
                } catch {
                    check();
                    cache = "unavailable";
                }
                check();
                timing.integrity = performance.now() - started;
            }
            let phaseStarted = performance.now();
            let module: WebAssembly.Module;
            if (cachedDigest !== undefined && latestModule && latestModule.key === key && latestModule.digest === cachedDigest) {
                module = latestModule.module;
                cache = "module-hit";
            } else {
                // Rust supplies JS-owned arrays before freeing its source buffers.
                module = await WebAssembly.compile(artifact.bytes as Uint8Array<ArrayBuffer>);
            }
            check();
            timing.compile = performance.now() - phaseStarted;
            phaseStarted = performance.now();
            const instance = await WebAssembly.instantiate(module, imports);
            check();
            timing.instantiate = performance.now() - phaseStarted;
            phaseStarted = performance.now();
            let groupStarted = phaseStarted;
            const dispatcher = instance.exports.dispatch;
            if (typeof dispatcher !== "function") throw new Error("missing compiled dispatcher");
            for (let slot = 0; slot < Math.max(regionCount, 1); slot++) {
                check();
                // The boxed host frame has PC 0x1000 and budget 0: no guest context is needed.
                if (dispatcher(frame, 0, slot) !== 2) throw new Error(`compiled region ${slot} failed budget-zero warmup`);
                check();
                if (performance.now() - groupStarted >= 4) {
                    await compilerTask();
                    check();
                    groupStarted = performance.now();
                }
            }
            check();
            timing.warmup = performance.now() - phaseStarted;
            latestModule = key !== undefined && digest !== undefined ? { key, digest, module } : undefined;
            if (key !== undefined && digest !== undefined && cachedDigest === undefined) {
                void storeArmCache({ bytes: artifact.bytes, manifest: artifact.manifest, digest }, key, deadline, signal);
            }
            return dispatcher;
        });
    } catch (error) {
        failed = true;
        stopped = error;
        throw error;
    } finally {
        const memoryRetained = memory.buffer.byteLength;
        imports = {};
        memory = undefined;
        console.info("ARM AOT prepared", {
            outcome: failed || signal.aborted ? String(stopped) : "ready",
            elapsedMs: performance.now() - started, phasesMs: timing, encodedSize: artifact.bytes.byteLength, regionCount, cache,
            hostMemoryBefore: memoryBefore, hostMemoryPeak: memoryRetained, hostMemoryRetained: memoryRetained,
        });
    }
}
