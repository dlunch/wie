import { WieWeb } from "@pkg";

import type { PlayerSession, ScaleFilter, SessionEvent } from "../backend";
import { AudioPlayer } from "../midi";
import { rustPanic } from "../panic";
import type { EmulatorHost } from "./tauri-host";

export interface WasmSession extends PlayerSession {
  suspend(suspended: boolean): void;
}

export interface PresentationOptions {
  scaleFilter: ScaleFilter;
  onError(error: unknown): void;
  signal: AbortSignal;
}

export const startWasmSession = async (
  filename: string,
  archive: Uint8Array,
  font: Uint8Array,
  onEvent: (event: SessionEvent) => void,
  signal: AbortSignal,
  scaleFilter: ScaleFilter,
  host?: EmulatorHost,
  isSuspended: () => boolean = () => document.hidden,
  checkStartup: () => boolean = () => true,
): Promise<WasmSession | undefined> => {
  if (signal.aborted || !checkStartup()) return;
  if (rustPanic) throw rustPanic;
  const canvas = document.createElement("canvas");
  canvas.id = "canvas";
  document.getElementById("player-output")!.prepend(canvas);
  const audio = host?.audio ?? new AudioPlayer();
  let emulator: WieWeb | undefined;
  const keys = new Set<string>();
  let running = true;
  let active = false;
  let handedOff = false;
  let audioDisposed = false;
  let startupError: unknown;
  let suspended = true;
  let preparing = true;
  let frame = 0;
  let redrawFrame = 0;
  let outputEvent = false;
  let dprQuery: MediaQueryList | undefined;

  const cleanup = () => {
    running = false;
    cancelAnimationFrame(frame);
    cancelAnimationFrame(redrawFrame);
    observer.disconnect();
    dprQuery?.removeEventListener("change", outputChanged);
    document.removeEventListener("visibilitychange", visibilityChanged);
    canvas.removeEventListener("webglcontextlost", contextLost);
    window.removeEventListener("pageshow", outputChanged);
    signal.removeEventListener("abort", cleanup);
    canvas.remove();
    // Aborting panics do not unwind Rust borrows; only JS resources can then be released.
    if (emulator && !rustPanic) {
      emulator.free();
      emulator = undefined;
      audioDisposed = true;
    } else if (!audioDisposed) {
      audio.dispose();
      audioDisposed = true;
    }
  };
  const fail = (error: unknown) => {
    if (!running || signal.aborted) return;
    const cause = rustPanic ?? error;
    cleanup();
    onEvent({ type: "error", message: String(cause), cause });
  };
  const requestRedraw = (external: boolean) => {
    if (!running || !emulator) return;
    outputEvent ||= external;
    if (redrawFrame) return;
    redrawFrame = requestAnimationFrame(() => {
      redrawFrame = 0;
      if (!running) return;
      const { width, height } = canvas.getBoundingClientRect();
      const dpr = window.devicePixelRatio;
      const external = outputEvent;
      outputEvent = false;
      try {
        if (emulator!.redraw(document.hidden ? 0 : Math.round(width * dpr), document.hidden ? 0 : Math.round(height * dpr), external)) {
          requestRedraw(false);
        }
      } catch (error) {
        fail(error);
      }
    });
  };
  const outputChanged = () => {
    dprQuery?.removeEventListener("change", outputChanged);
    dprQuery = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
    dprQuery.addEventListener("change", outputChanged);
    requestRedraw(true);
  };
  const observer = new ResizeObserver(outputChanged);
  const contextLost = () => {
    const error = new Error("WebGL presentation context lost");
    startupError ??= error;
    if (active) fail(error);
  };
  const visibilityChanged = () => {
    if (active) session.suspend(isSuspended());
    requestRedraw(true);
  };
  observer.observe(canvas);
  outputChanged();
  document.addEventListener("visibilitychange", visibilityChanged);
  canvas.addEventListener("webglcontextlost", contextLost);
  window.addEventListener("pageshow", outputChanged);
  signal.addEventListener("abort", cleanup, { once: true });

  const session: WasmSession = {
    async key(key, pressed) {
      if (!running || preparing || suspended) return;
      if (pressed && !keys.has(key)) {
        emulator!.key_down(key);
        keys.add(key);
      } else if (!pressed && keys.delete(key)) {
        emulator!.key_up(key);
      }
    },
    async releaseKeys() {
      if (!running) return;
      for (const key of keys) emulator!.key_up(key);
      keys.clear();
    },
    suspend(paused) {
      if (!running) return;
      suspended = paused;
      for (const key of keys) emulator!.key_up(key);
      keys.clear();
      cancelAnimationFrame(frame);
      onEvent({ type: "suspended", suspended });
      if (!suspended && running) {
        requestRedraw(true);
        frame = requestAnimationFrame(update);
      }
    },
    async stop() {
      if (!running) return;
      cleanup();
      onEvent({ type: "stopped", failed: false });
    },
  };

  const update = () => {
    if (!running || suspended) return;
    try {
      if (emulator!.update()) requestRedraw(false);
      else if (!outputEvent) {
        cancelAnimationFrame(redrawFrame);
        redrawFrame = 0;
      }
      if (emulator!.is_exited()) {
        void session.stop();
        return;
      }
      if (preparing && !emulator!.is_preparing()) {
        preparing = false;
        onEvent({ type: "ready" });
      }
      if (running && !suspended) frame = requestAnimationFrame(update);
    } catch (error) {
      fail(error);
    }
  };
  try {
    emulator = await WieWeb.create(filename, archive, canvas, font, audio, host, {
      scaleFilter,
      signal,
      onError(error: unknown) {
        if (!running || signal.aborted) return;
        startupError ??= error;
        if (active) fail(error);
      },
    });
    if (startupError !== undefined) throw startupError;
    if (!running || signal.aborted || !checkStartup()) return;
    active = true;
    suspended = isSuspended();
    onEvent({ type: "suspended", suspended });
    if (!running || signal.aborted) return;
    requestRedraw(true);
    if (!suspended) frame = requestAnimationFrame(() => {
      if (running && !suspended) frame = requestAnimationFrame(update);
    });
    handedOff = true;
    return session;
  } catch (error) {
    if (startupError !== undefined) throw startupError;
    if (!signal.aborted) throw rustPanic ?? error;
  } finally {
    if (!handedOff) cleanup();
  }
};
