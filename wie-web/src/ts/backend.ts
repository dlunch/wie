import { isTauri } from "@tauri-apps/api/core";

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

export const initializeWie = async (): Promise<Wie> => {
  const backend = isTauri()
    ? await import("./backend/tauri")
    : await import("./backend/browser");

  return backend.initializeWie();
};
