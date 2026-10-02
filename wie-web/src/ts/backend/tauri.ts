import { Channel, invoke } from "@tauri-apps/api/core";

import type { LibraryApp, SessionEvent, Wie } from "../backend";

type NativeApp = Omit<LibraryApp, "icon"> & { icon: number[] | null };

export const initializeWie = async (): Promise<Wie> => {
  return {
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
