import { Channel, invoke } from "@tauri-apps/api/core";

import type { LibraryApp, PlayerSession, SessionEvent, Settings, Wie } from "../backend";
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
    async startApp(id, onEvent, signal) {
      if (signal.aborted) return;
      if (runtime === "wasm") {
        let host: TauriHost | undefined;
        let session: WasmSession | undefined;
        let lifecycle: HostLifecycle | undefined;
        let startupError: unknown;
        let ended = false;
        let handedOff = false;
        let receivedClock!: () => void;
        const clockReady = new Promise<void>(resolve => { receivedClock = resolve; });
        signal.addEventListener("abort", receivedClock, { once: true });
        const checkStartup = () => {
          if (startupError !== undefined) throw startupError;
          return !ended && !signal.aborted;
        };
        const report = (event: SessionEvent) => {
          if (ended || signal.aborted) return;
          if (event.type === "error" || event.type === "stopped") ended = true;
          onEvent(event);
        };
        const events = new Channel<HostEvent>(event => {
          if (ended || signal.aborted) return;
          if (event.type === "lifecycle") {
            lifecycle = event;
            host?.lifecycle(event);
            session?.suspend(event.suspended);
            receivedClock();
          } else if (event.type === "warning") {
            report(event);
          } else {
            receivedClock();
            report(event);
          }
        });
        try {
          const started = await invoke<{ sessionId: number; filename: string; bytes: number[] } | null>("start_app", { id, events });
          if (started === null) {
            report({ type: "stopped", failed: true });
            return;
          }
          host = new TauriHost(started.sessionId, error => {
            startupError ??= error;
            if (session) report({ type: "error", message: String(error), cause: error });
          });
          if (!checkStartup()) return;
          await clockReady;
          if (!checkStartup()) return;
          host.lifecycle(lifecycle!);
          const [{ startWasmSession }, fontResponse, settings] = await Promise.all([
            import("./wasm"),
            fetch(new URL("../../../../assets/neodgm.ttf", import.meta.url)),
            invoke<Settings>("read_settings"),
          ]);
          if (!checkStartup()) return;
          if (!fontResponse.ok) throw new Error(`Failed to load font: ${fontResponse.status} ${fontResponse.statusText}`);
          const font = new Uint8Array(await fontResponse.arrayBuffer());
          if (!checkStartup()) return;
          const startedSession = await startWasmSession(started.filename, new Uint8Array(started.bytes), font, report,
            signal, settings.scaleFilter, host, () => lifecycle!.suspended, checkStartup);
          session = startedSession;
          if (!startedSession || !checkStartup()) return;
          let stopping: Promise<void> | undefined;
          handedOff = true;
          return {
            key: startedSession.key,
            releaseKeys: startedSession.releaseKeys,
            stop() {
              stopping ??= (async () => {
                try {
                  await startedSession.stop();
                } finally {
                  await host!.stop();
                }
              })();
              return stopping;
            },
          };
        } catch (error) {
          if (!signal.aborted || startupError !== undefined) throw error;
        } finally {
          signal.removeEventListener("abort", receivedClock);
          if (!handedOff) {
            try {
              await session?.stop();
            } finally {
              await host?.stop();
            }
          }
        }
        return;
      }
      let ended = false;
      let stopping: Promise<void> | undefined;
      const events = new Channel<Exclude<SessionEvent, { type: "error" }>>(event => {
        if (ended || signal.aborted) return;
        if (event.type === "stopped") ended = true;
        onEvent(event);
      });
      let sessionId: number | null;
      try {
        sessionId = await invoke<number | null>("start_app", { id, events });
      } catch (error) {
        if (signal.aborted) return;
        throw error;
      }
      if (sessionId === null) {
        if (!signal.aborted) onEvent({ type: "stopped", failed: true });
        return;
      }
      let commands = Promise.resolve();

      const send = (command: string, args: Record<string, unknown> = {}) => {
        const request = commands.then(() => {
          if (!ended) return invoke<void>(command, { sessionId, ...args });
        });
        // A rejected input must not prevent cleanup; the caller still receives the error.
        commands = request.catch(() => {});
        return request;
      };

      const session: PlayerSession = {
        key(key, pressed) {
          if (stopping) return Promise.resolve();
          return send("key_event", { key, pressed });
        },
        releaseKeys() {
          if (stopping) return Promise.resolve();
          return send("release_keys");
        },
        stop() {
          stopping ??= send("stop_app").then(() => { ended = true; });
          return stopping;
        },
      };
      if (signal.aborted || ended) {
        await session.stop();
        return;
      }
      return session;
    },
  };
};
