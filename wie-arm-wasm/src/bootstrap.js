export function createCompilerWorker() {
    return new Worker(new URL("@wie-arm-worker", import.meta.url), { type: "module" });
}

export function executeRegion(region, frame, context) {
    const exit = region(frame, context);
    // Reject non-numbers before the Wasm import can coerce them to valid exits.
    return typeof exit === "number" ? exit : NaN;
}
