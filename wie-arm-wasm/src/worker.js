import("@wie-arm-compiler").then(({ compile_request }) => {
    self.onmessage = async ({ data: { request, payload } }) => {
        try {
            if (!(payload instanceof Uint8Array)) throw new TypeError("compiler payload must be bytes");
            const artifact = compile_request(payload);
            const module = await WebAssembly.compile(artifact.bytes);
            self.postMessage({ request, module, manifest: artifact.manifest, encodedSize: artifact.bytes.length });
        } catch (error) {
            self.postMessage({ request, error: String(error) });
        }
    };
    self.postMessage({ ready: true });
}).catch(error => {
    self.postMessage({ initializationError: String(error) });
});
