interface ArmCacheLookup {
    key?: Uint8Array;
    artifact?: Uint8Array;
    digest?: Uint8Array;
}

let latestModule: { key: string; digest: Uint8Array; module: WebAssembly.Module } | undefined;

async function accessArmCache(key: string, record: Uint8Array | undefined): Promise<unknown> {
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

export async function loadArmCache(input: Uint8Array): Promise<ArmCacheLookup> {
    let outcome = "miss";
    let key: Uint8Array | undefined;
    try {
        key = new Uint8Array(await crypto.subtle.digest("SHA-256", input as Uint8Array<ArrayBuffer>));
        const storageKey = Array.from(key, byte => byte.toString(16).padStart(2, "0")).join("");
        const record = await accessArmCache(storageKey, undefined);
        if (record === undefined) return { key };
        outcome = "corrupt";
        if (!(record instanceof Uint8Array) || record.length < 32) return { key };
        const artifact = record.subarray(0, -32);
        const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", artifact as Uint8Array<ArrayBuffer>));
        if (!digest.every((byte, index) => byte === record[artifact.length + index])) return { key };
        outcome = "persistent-hit";
        return { key, artifact, digest };
    } catch {
        outcome = "unavailable";
        return { key };
    } finally {
        console.info("ARM AOT cache lookup", outcome);
    }
}

// This detached task owns only cache data, never an instance, imports or a warmup frame.
async function storeArmCache(record: Uint8Array, key: string, digest: Uint8Array): Promise<void> {
    let outcome = "stored";
    try {
        const bytes = new Uint8Array(record.length + digest.length);
        bytes.set(record);
        bytes.set(digest, record.length);
        await accessArmCache(key, bytes);
    } catch {
        outcome = "failed";
    }
    console.info("ARM AOT cache store", outcome);
}

export function yieldToMainThread(): Promise<void> {
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
    bytes: Uint8Array, record: Uint8Array | undefined, keyBytes: Uint8Array | undefined, cachedDigest: Uint8Array | undefined, imports: WebAssembly.Imports,
    frame: number, regionCount: number,
): Promise<Function> {
    let outcome = cachedDigest === undefined ? "miss" : "persistent-hit";
    try {
        const key = keyBytes && Array.from(keyBytes, byte => byte.toString(16).padStart(2, "0")).join("");
        let digest = cachedDigest;
        if (key !== undefined && record !== undefined && digest === undefined) {
            try {
                digest = new Uint8Array(await crypto.subtle.digest("SHA-256", record as Uint8Array<ArrayBuffer>));
            } catch {
                outcome = "unavailable";
            }
        }
        let module: WebAssembly.Module;
        const previous = latestModule;
        if (cachedDigest !== undefined && previous && previous.key === key &&
            cachedDigest.every((byte, index) => byte === previous.digest[index])) {
            module = previous.module;
            outcome = "module-hit";
        } else {
            // Rust supplies JS-owned arrays before freeing its source buffers.
            module = await WebAssembly.compile(bytes as Uint8Array<ArrayBuffer>);
        }
        const instance = await WebAssembly.instantiate(module, imports);
        let groupStarted = performance.now();
        const dispatcher = instance.exports.dispatch;
        if (typeof dispatcher !== "function") throw new Error("missing compiled dispatcher");
        for (let slot = 0; slot < Math.max(regionCount, 1); slot++) {
            // The boxed host frame has PC and return address 0x1000: no guest context is needed.
            if (dispatcher(frame, 0, slot) !== 3) throw new Error(`compiled region ${slot} failed return-boundary warmup`);
            if (slot + 1 < regionCount && performance.now() - groupStarted >= 4) {
                await yieldToMainThread();
                groupStarted = performance.now();
            }
        }
        if (key !== undefined && digest !== undefined) latestModule = { key, digest, module };
        if (key !== undefined && record !== undefined && digest !== undefined && cachedDigest === undefined) {
            void storeArmCache(record, key, digest);
        }
        return dispatcher;
    } catch (error) {
        outcome = String(error);
        throw error;
    } finally {
        console.info("ARM AOT prepared", outcome);
    }
}
