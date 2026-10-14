import { WieWeb } from "@pkg";

import type { PlayerSession, SessionEvent } from "../backend";
import type { EmulatorHost } from "./tauri-host";

export interface WasmSession extends PlayerSession {
  suspend(suspended: boolean): void;
}

export const startWasmSession = (
  filename: string,
  archive: Uint8Array,
  font: Uint8Array,
  enableAot: boolean,
  onEvent: (event: SessionEvent) => void,
  host?: EmulatorHost,
  suspended = false,
): WasmSession => {
  const canvas = document.getElementById("canvas") as HTMLCanvasElement;
  canvas.width = 240;
  canvas.height = 320;
  const emulator = new WieWeb(filename, archive, canvas, font, enableAot, host);
  const keys = new Set<string>();
  let running = true;
  let preparing = true;
  let frame: number;

  const session: WasmSession = {
    async key(key, pressed) {
      if (!running || preparing || suspended) return;
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
    suspend(paused) {
      if (!running) return;
      suspended = paused;
      for (const key of keys) emulator.key_up(key);
      keys.clear();
      cancelAnimationFrame(frame);
      onEvent({ type: "suspended", suspended });
      if (!suspended) frame = requestAnimationFrame(update);
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
    if (!running || suspended) return;
    try {
      emulator.update();
      if (emulator.is_exited()) {
        void session.stop();
        return;
      }
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
  onEvent({ type: "suspended", suspended });
  if (!suspended) frame = requestAnimationFrame(() => { frame = requestAnimationFrame(update); });
  return session;
};
