export function createCompilerWorker() {
    return new Worker(new URL("@wie-arm-worker", import.meta.url), { type: "module" });
}
