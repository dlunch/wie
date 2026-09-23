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
    deadline: number, work: (check: () => void) => Promise<T>,
): Promise<T> {
    let settled = false;
    const check = () => {
        if (settled || performance.now() >= deadline) throw new Error("ARM AOT preparation timed out");
    };
    let interrupt!: (error: unknown) => void;
    const interrupted = new Promise<never>((_, reject) => { interrupt = reject; });
    const timeout = setTimeout(() => {
        // Timer delay rounding can settle the race before the absolute deadline.
        settled = true;
        interrupt(new Error("ARM AOT preparation timed out"));
    }, Math.max(0, deadline - performance.now()));
    try {
        return await Promise.race([interrupted, work(check)]);
    } finally {
        settled = true;
        clearTimeout(timeout);
    }
}

async function sha256(bytes: Uint8Array): Promise<string> {
    const digest = await crypto.subtle.digest("SHA-256", bytes as Uint8Array<ArrayBuffer>);
    return Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, "0")).join("");
}

async function artifactDigest(artifact: ArmArtifactBytes, key: string): Promise<string> {
    const bytes = await sha256(artifact.bytes);
    const manifest = await sha256(artifact.manifest);
    return sha256(new TextEncoder().encode(JSON.stringify([key, bytes, manifest])));
}

async function accessArmCache(key: string, record: ArmCacheRecord | undefined): Promise<unknown> {
    let db: IDBDatabase | undefined;
    const opening = indexedDB.open("wie_arm_aot");
    try {
        return await new Promise((resolve, reject) => {
            opening.onupgradeneeded = () => {
                opening.result.createObjectStore("artifacts");
            };
            opening.onerror = () => reject(opening.error);
            opening.onblocked = () => reject(new Error("ARM AOT cache blocked"));
            opening.onsuccess = () => {
                db = opening.result;
                try {
                    const current = db.transaction("artifacts", record ? "readwrite" : "readonly");
                    const store = current.objectStore("artifacts");
                    const request = record ? store.put(record, key) : store.get(key);
                    current.oncomplete = () => resolve(request.result);
                    current.onerror = current.onabort = () => {
                        reject(current.error);
                    };
                } catch (error) {
                    db.close();
                    reject(error);
                }
            };
        });
    } finally {
        // A blocked request can succeed after this operation has already returned.
        opening.onsuccess = () => opening.result.close();
        db?.close();
    }
}

export async function loadArmCache(input: Uint8Array, version: number, deadline: number): Promise<ArmCacheLookup> {
    const started = performance.now();
    const timing: Record<string, number> = {};
    let outcome = "miss";
    let key: string | undefined;
    try {
        return await duringPreparation(deadline, async () => {
            try {
                key = `${version}:${await sha256(input)}`;
                timing.keying = performance.now() - started;
                let phaseStarted = performance.now();
                const record = await accessArmCache(key, undefined);
                timing.read = performance.now() - phaseStarted;
                phaseStarted = performance.now();
                if (record === undefined) return { key };
                outcome = "corrupt";
                if (typeof record !== "object" || record === null ||
                    !("bytes" in record) || !(record.bytes instanceof Uint8Array) ||
                    !("manifest" in record) || !(record.manifest instanceof Uint8Array) ||
                    !("digest" in record) || typeof record.digest !== "string") return { key };
                const artifact = { bytes: record.bytes, manifest: record.manifest, digest: record.digest };
                const digest = await artifactDigest(artifact, key);
                timing.integrity = performance.now() - phaseStarted;
                if (digest !== artifact.digest) return { key };
                outcome = "persistent-hit";
                return { key, artifact };
            } catch {
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
async function storeArmCache(record: ArmCacheRecord, key: string, deadline: number): Promise<void> {
    const started = performance.now();
    let outcome = "stored";
    try {
        await duringPreparation(deadline, () => accessArmCache(key, record));
    } catch {
        outcome = performance.now() >= deadline ? "skipped" : "failed";
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
    frame: number, regionCount: number, deadline: number,
): Promise<Function> {
    let stopped: unknown;
    let failed = false;
    const started = performance.now();
    const timing: Record<string, number> = {};
    let cache = cachedDigest === undefined ? "miss" : "persistent-hit";
    let memory: WebAssembly.Memory | undefined = imports.wie.memory as WebAssembly.Memory;
    const memoryBefore = memory.buffer.byteLength;
    try {
        return await duringPreparation(deadline, async check => {
            let digest = cachedDigest;
            if (key !== undefined && digest === undefined) {
                try {
                    digest = await artifactDigest(artifact, key);
                } catch {
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
            // Promise.race does not cancel work: Rust may have freed the warmup frame after a timeout.
            check();
            timing.instantiate = performance.now() - phaseStarted;
            phaseStarted = performance.now();
            let groupStarted = phaseStarted;
            const dispatcher = instance.exports.dispatch;
            if (typeof dispatcher !== "function") throw new Error("missing compiled dispatcher");
            for (let slot = 0; slot < Math.max(regionCount, 1); slot++) {
                // The boxed host frame has PC and return address 0x1000: no guest context is needed.
                if (dispatcher(frame, 0, slot) !== 3) throw new Error(`compiled region ${slot} failed return-boundary warmup`);
                if (slot + 1 < regionCount && performance.now() - groupStarted >= 4) {
                    await compilerTask();
                    check();
                    groupStarted = performance.now();
                }
            }
            timing.warmup = performance.now() - phaseStarted;
            latestModule = key !== undefined && digest !== undefined ? { key, digest, module } : undefined;
            if (key !== undefined && digest !== undefined && cachedDigest === undefined) {
                void storeArmCache({ bytes: artifact.bytes, manifest: artifact.manifest, digest }, key, deadline);
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
            outcome: failed ? String(stopped) : "ready",
            elapsedMs: performance.now() - started, phasesMs: timing, encodedSize: artifact.bytes.byteLength, regionCount, cache,
            hostMemoryBefore: memoryBefore, hostMemoryPeak: memoryRetained, hostMemoryRetained: memoryRetained,
        });
    }
}
