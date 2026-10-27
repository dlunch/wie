(() => {
  // Wry injects Android scripts into every frame; only the main frame may receive the IPC key.
  if (window.top !== window) return;

  const invokeKey = __INVOKE_KEY__;
  Object.defineProperty(window.__TAURI_INTERNALS__, "postMessage", {
    value(message) {
      const envelope = {
        ...message,
        options: { ...message.options, customProtocolIpcBlocked: true },
        __TAURI_INVOKE_KEY__: invokeKey,
      };
      const serialized = JSON.stringify(envelope, (_key, value) => {
        if (value && typeof value.__TAURI_TO_IPC_KEY__ === "function") return value.__TAURI_TO_IPC_KEY__();
        if (value instanceof Map) return Object.fromEntries(value);
        if (value instanceof ArrayBuffer) return Array.from(new Uint8Array(value));
        if (value instanceof Uint8Array) return Array.from(value);
        return value;
      });
      window.ipc.postMessage(serialized);
    },
  });
})();
