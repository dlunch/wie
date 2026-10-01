import { Channel, invoke, isTauri } from "@tauri-apps/api/core";

export interface LibraryApp {
  id: string;
  title: string;
  filename: string;
  icon?: Blob;
  addedAt: number;
}

export interface Settings {
  midiVolume: number;
  pcmVolume: number;
  helpDismissed: boolean;
  welcomeSeen: boolean;
  enableWasmAot?: boolean;
}

export type SessionEvent =
  | { type: "ready" }
  | { type: "warning"; message: string }
  | { type: "stopped" }
  | { type: "error"; message: string };

export interface PlayerSession {
  key(key: string, pressed: boolean): Promise<void>;
  releaseKeys(): Promise<void>;
  stop(): Promise<void>;
}

export interface Wie {
  listApps(): Promise<LibraryApp[]>;
  importApp(file: File): Promise<void>;
  deleteApp(id: string): Promise<void>;
  readSettings(): Promise<Settings>;
  writeSettings(settings: Settings): Promise<void>;
  startGame(id: string, onEvent: (event: SessionEvent) => void): Promise<PlayerSession>;
}

type NativeApp = Omit<LibraryApp, "icon"> & { icon: number[] | null };

export const initializeWie = async (): Promise<Wie> => {
  if (isTauri()) {
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
  }

  const [{ extractAppMetadata, WieWeb }, { AppLibraryStore }, { configStore }, { setMasterVolume, setPcmVolume }, Sentry] = await Promise.all([
    import("@pkg"),
    import("./app_library_store"),
    import("./config_store"),
    import("./midi"),
    import("@sentry/browser"),
  ]);
  Sentry.init({
    dsn: "https://fa9187d6bd7dd43ae621f26d33641f81@o106536.ingest.us.sentry.io/4512048969678848",
  });
  const store = await AppLibraryStore.open();

  const backend: Wie = {
    listApps() {
      return store.list();
    },
    async importApp(file) {
      const archive = new Uint8Array(await file.arrayBuffer());
      const extracted = extractAppMetadata(file.name, archive);
      try {
        const icon = extracted.icon;
        const app: LibraryApp = {
          id: extracted.id,
          title: extracted.title,
          filename: file.name,
          addedAt: Date.now(),
          icon: icon.length ? new Blob([new Uint8Array(icon).buffer]) : undefined,
        };
        try {
          await store.add(app, archive);
        } catch (error) {
          if (error instanceof DOMException && error.name === "ConstraintError") {
            throw new Error(`이미 라이브러리에 추가된 앱입니다: ${app.title}`);
          }
          throw error;
        }
      } finally {
        extracted.free();
      }
    },
    deleteApp(id) {
      return store.delete(id);
    },
    async readSettings() {
      return {
        midiVolume: configStore.get("midiVolume"),
        pcmVolume: configStore.get("pcmVolume"),
        enableWasmAot: configStore.get("enableWasmAot"),
        helpDismissed: configStore.get("helpDismissed"),
        welcomeSeen: configStore.get("welcomeSeen"),
      };
    },
    async writeSettings(settings) {
      configStore.set("midiVolume", settings.midiVolume);
      configStore.set("pcmVolume", settings.pcmVolume);
      configStore.set("enableWasmAot", settings.enableWasmAot ?? false);
      configStore.set("helpDismissed", settings.helpDismissed);
      configStore.set("welcomeSeen", settings.welcomeSeen);
      setMasterVolume(settings.midiVolume);
      setPcmVolume(settings.pcmVolume);
    },
    async startGame(id, onEvent) {
      const [app, archive, settings, fontResponse] = await Promise.all([
        store.getApp(id),
        store.getArchive(id),
        backend.readSettings(),
        fetch(new URL("../../../assets/neodgm.ttf", import.meta.url)),
      ]);
      if (!app || !archive) throw new Error("저장된 앱 파일을 찾을 수 없습니다.");
      if (!fontResponse.ok) throw new Error(`Failed to load font: ${fontResponse.status} ${fontResponse.statusText}`);
      const fontData = new Uint8Array(await fontResponse.arrayBuffer());
      const canvas = document.getElementById("canvas") as HTMLCanvasElement;
      canvas.width = 240;
      canvas.height = 320;
      setMasterVolume(settings.midiVolume);
      setPcmVolume(settings.pcmVolume);
      const emulator = new WieWeb(app.filename, archive, canvas, fontData, settings.enableWasmAot ?? false);
      const keys = new Set<string>();
      let running = true;
      let preparing = true;
      let frame: number;

      const session: PlayerSession = {
        async key(key, pressed) {
          if (!running || preparing) return;
          if (pressed && !keys.has(key)) {
            emulator.key_down(key);
            keys.add(key);
          } else if (!pressed && keys.delete(key)) {
            emulator.key_up(key);
          }
        },
        async releaseKeys() {
          if (!running) return;
          for (const key of keys) emulator.key_up(key);
          keys.clear();
        },
        async stop() {
          if (!running) return;
          running = false;
          cancelAnimationFrame(frame);
          emulator.free();
          onEvent({ type: "stopped" });
        },
      };

      const update = () => {
        if (!running) return;
        try {
          emulator.update();
          if (preparing && !emulator.is_preparing()) {
            preparing = false;
            onEvent({ type: "ready" });
          }
          frame = requestAnimationFrame(update);
        } catch (error) {
          running = false;
          emulator.free();
          onEvent({ type: "error", message: String(error) });
        }
      };
      frame = requestAnimationFrame(() => { frame = requestAnimationFrame(update); });
      return session;
    },
  };
  return backend;
};
