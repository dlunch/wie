export async function compileArm(
    bytes: Uint8Array<ArrayBuffer>, imports: WebAssembly.Imports,
    frame: number, names: string[], deadline: number, signal: AbortSignal,
): Promise<Function[]> {
    let stopped: unknown;
    let failed = false;
    let taskTimer: ReturnType<typeof setTimeout> | undefined;
    let wake: (() => void) | undefined;
    const functions: Function[] = [];
    const started = performance.now();
    const timing: Record<string, number> = {};
    let memory: WebAssembly.Memory | undefined = imports.wie.memory as WebAssembly.Memory;
    const memoryBefore = memory.buffer.byteLength;
    const encodedSize = bytes.length;
    const regionCount = names.length;
    const check = () => {
        if (stopped) throw stopped;
        signal.throwIfAborted();
        if (performance.now() >= deadline) throw new Error("ARM AOT preparation timed out");
    };
    let interrupt!: (error: unknown) => void;
    const interrupted = new Promise<never>((_, reject) => { interrupt = reject; });
    const abort = () => {
        stopped = signal.reason;
        interrupt(stopped);
    };
    signal.addEventListener("abort", abort, { once: true });
    const timeout = setTimeout(() => {
        stopped = new Error("ARM AOT preparation timed out");
        interrupt(stopped);
    }, Math.max(0, deadline - performance.now()));

    try {
        return await Promise.race([interrupted, (async () => {
            check();
            const compiling = WebAssembly.compile(bytes);
            bytes = new Uint8Array(0);
            const module = await compiling;
            check();
            timing.compile = performance.now() - started;
            let phaseStarted = performance.now();
            const instance = await WebAssembly.instantiate(module, imports);
            check();
            timing.instantiate = performance.now() - phaseStarted;
            phaseStarted = performance.now();
            let groupStarted = phaseStarted;
            for (const name of names) {
                check();
                const fn = instance.exports[name];
                if (typeof fn !== "function") throw new Error(`missing compiled export ${name}`);
                // The boxed host frame has PC 0x1000 and budget 0: no guest context is needed.
                if (fn(frame, 0) !== 2) throw new Error(`compiled export ${name} failed budget-zero warmup`);
                functions.push(fn);
                check();
                if (performance.now() - groupStarted >= 4) {
                    await new Promise<void>(resolve => {
                        wake = resolve;
                        taskTimer = setTimeout(resolve, 0);
                    });
                    wake = undefined;
                    taskTimer = undefined;
                    check();
                    groupStarted = performance.now();
                }
            }
            check();
            timing.warmup = performance.now() - phaseStarted;
            return functions;
        })()]);
    } catch (error) {
        failed = true;
        stopped = error;
        functions.length = 0;
        throw error;
    } finally {
        clearTimeout(timeout);
        clearTimeout(taskTimer);
        signal.removeEventListener("abort", abort);
        wake?.();
        names.length = 0;
        bytes = new Uint8Array(0);
        const memoryRetained = memory.buffer.byteLength;
        imports = {};
        memory = undefined;
        console.info("ARM AOT prepared", {
            outcome: failed || signal.aborted ? String(stopped) : "ready",
            elapsedMs: performance.now() - started, phasesMs: timing, encodedSize, regionCount,
            hostMemoryBefore: memoryBefore, hostMemoryPeak: memoryRetained, hostMemoryRetained: memoryRetained,
        });
    }
}
