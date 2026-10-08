import { Channel, invoke } from "@tauri-apps/api/core";

import type { LibraryApp, SessionEvent, Wie } from "../backend";
import { TauriHost, type HostEvent, type HostLifecycle } from "./tauri-host";
import type { WasmSession } from "./wasm";

type NativeApp = Omit<LibraryApp, "icon"> & { icon: number[] | null };

export const initializeWie = async (): Promise<Wie> => {
  const runtime = await invoke<"wasm" | "native">("runtime_kind");
  return {
    rendering: runtime === "wasm" ? "canvas" : "native",
    async listApps() {
      return (await invoke<NativeApp[]>("list_apps")).map(app => ({
        ...app,
        icon: app.icon?.length ? new Blob([Uint8Array.from(app.icon).buffer]) : undefined,
      }));
    },
    async importApp(file) {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      await invoke("import_app", { filename: file.name, bytes });
    },
    deleteApp(id) {
      return invoke("delete_app", { id });
    },
    readSettings() {
      return invoke("read_settings");
    },
    writeSettings(settings) {
      return invoke("write_settings", { settings });
    },
    async startGame(id, onEvent) {
      if (runtime === "wasm") {
        let host: TauriHost | undefined;
        let session: WasmSession | undefined;
        let lifecycle: HostLifecycle | undefined;
        let startupError: unknown;
        let ended = false;
        let receivedClock!: () => void;
        const clockReady = new Promise<void>(resolve => { receivedClock = resolve; });
        const report = (event: SessionEvent) => {
          if (ended) return;
          if (event.type === "error" || event.type === "stopped") ended = true;
          onEvent(event);
        };
        const events = new Channel<HostEvent>(event => {
          if (event.type === "lifecycle") {
            lifecycle = event;
            host?.lifecycle(event);
            session?.suspend(event.suspended);
            receivedClock();
          } else if (event.type === "warning") {
            report(event);
          } else {
            startupError = event.type === "error" ? new Error(event.message) : new Error("Game stopped during initialization");
            receivedClock();
            if (session) report(event);
          }
        });
        const started = await invoke<{ sessionId: number; filename: string; bytes: number[] }>("start_web_game", { id, events });
        host = new TauriHost(started.sessionId, error => {
          startupError ??= error;
          if (session) report({ type: "error", message: String(error) });
        });
        try {
          await clockReady;
          if (startupError !== undefined) throw startupError;
          host.lifecycle(lifecycle!);
          const [{ startWasmSession }, fontResponse] = await Promise.all([
            import("./wasm"),
            fetch(new URL("../../../../assets/neodgm.ttf", import.meta.url)),
          ]);
          if (!fontResponse.ok) throw new Error(`Failed to load font: ${fontResponse.status} ${fontResponse.statusText}`);
          const font = new Uint8Array(await fontResponse.arrayBuffer());
          if (startupError !== undefined) throw startupError;
          const startedSession = startWasmSession(started.filename, new Uint8Array(started.bytes), font, true, report, host, lifecycle!.suspended);
          session = startedSession;
          let stopping: Promise<void> | undefined;
          return {
            key: startedSession.key,
            releaseKeys: startedSession.releaseKeys,
            stop() {
              stopping ??= (async () => {
                try {
                  await startedSession.stop();
                } finally {
                  await host.stop();
                }
              })();
              return stopping;
            },
          };
        } catch (error) {
          await host.stop().catch(() => {});
          throw error;
        }
      }
      let ended = false;
      let stopping: Promise<void> | undefined;
      const events = new Channel<SessionEvent>(event => {
        if (ended) return;
        if (event.type === "stopped" || event.type === "error") ended = true;
        onEvent(event);
      });
      const sessionId = await invoke<number>("start_game", { id, events });
      let commands = Promise.resolve();

      const send = (command: string, args: Record<string, unknown> = {}) => {
        const request = commands.then(() => {
          if (!ended) return invoke<void>(command, { sessionId, ...args });
        });
        // A rejected input must not prevent cleanup; the caller still receives the error.
        commands = request.catch(() => {});
        return request;
      };

      return {
        key(key, pressed) {
          if (stopping) return Promise.resolve();
          return send("key_event", { key, pressed });
        },
        releaseKeys() {
          if (stopping) return Promise.resolve();
          return send("release_keys");
        },
        stop() {
          stopping ??= send("stop_game").then(() => { ended = true; });
          return stopping;
        },
      };
    },
  };
};
