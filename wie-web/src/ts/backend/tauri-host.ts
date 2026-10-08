import { invoke } from "@tauri-apps/api/core";

export type StorageRequest =
  | { op: "fileExists" | "fileSize"; aid: string; path: string }
  | { op: "fileRead"; aid: string; path: string; offset: number; count: number }
  | { op: "fileWrite"; aid: string; path: string; offset: number; data: number[] }
  | { op: "fileTruncate"; aid: string; path: string; length: number }
  | { op: "dbOpen" | "dbExists" | "dbDelete" | "recordNextId" | "recordIds"; pid: string; name: string }
  | { op: "dbUsage"; pid: string }
  | { op: "recordGet" | "recordDelete"; pid: string; name: string; id: number }
  | { op: "recordAdd"; pid: string; name: string; data: number[] }
  | { op: "recordSet"; pid: string; name: string; id: number; data: number[] };

export type StorageResult = null | boolean | number | number[];
export type HostLifecycle = { type: "lifecycle"; suspended: boolean; guestTimeMs: number };
export type HostEvent = HostLifecycle
  | { type: "warning"; message: string }
  | { type: "error"; message: string }
  | { type: "stopped" };

type AudioEvent =
  | [time: number, kind: "midi", data: Uint8Array]
  | [time: number, kind: "wave", channels: number, samplingRate: number, samples: Int16Array];

export interface EmulatorHost {
  storage(request: StorageRequest): Promise<StorageResult>;
  now(): number;
  vibrate(durationMs: number, intensity: number): void;
  audio: {
    play(handle: number, duration: number, events: AudioEvent[], repeat: boolean): void;
    stop(handle: number): void;
    dispose(): void;
  };
}

export class TauriHost implements EmulatorHost {
  private readonly sessionId: number;
  private readonly onError: (error: unknown) => void;
  private commands: Promise<void> = Promise.resolve();
  private closing = false;
  private stopping?: Promise<void>;
  private failure?: unknown;
  private guestTime = 0;
  private sampledAt = 0;
  private lastTime = 0;
  private suspended = true;

  constructor(sessionId: number, onError: (error: unknown) => void) {
    this.sessionId = sessionId;
    this.onError = onError;
  }

  private send<T>(command: string, args: Record<string, unknown>): Promise<T> {
    if (this.closing) return Promise.reject(new Error("Game session has stopped"));
    if (this.failure !== undefined) return Promise.reject(this.failure);
    const request = this.commands.then(() => invoke<T>(command, { sessionId: this.sessionId, ...args }));
    this.commands = request.then(() => {}, error => {
      const first = this.failure === undefined;
      this.failure ??= error;
      if (first && !this.closing) this.onError(error);
    });
    return request;
  }

  storage(request: StorageRequest): Promise<StorageResult> {
    return this.send("guest_storage", { request });
  }

  readonly audio = {
    play: (handle: number, duration: number, events: AudioEvent[], repeat: boolean): void => {
      const wireEvents = events.map(event => event[1] === "midi"
        ? { time: event[0], kind: "midi", data: Array.from(event[2]) }
        : { time: event[0], kind: "wave", channels: event[2], samplingRate: event[3], samples: Array.from(event[4]) });
      void this.send("guest_audio", { command: { type: "play", handle, duration, events: wireEvents, repeat } }).catch(() => {});
    },
    stop: (handle: number): void => {
      void this.send("guest_audio", { command: { type: "stop", handle } }).catch(() => {});
    },
    dispose: (): void => { this.closing = true; },
  };

  vibrate(durationMs: number, intensity: number): void {
    void this.send("guest_vibrate", { durationMs, intensity }).catch(() => {});
  }

  lifecycle(event: HostLifecycle): void {
    this.guestTime = Math.max(this.lastTime, event.guestTimeMs);
    this.sampledAt = performance.now();
    this.suspended = event.suspended;
  }

  now(): number {
    this.lastTime = Math.max(this.lastTime, this.guestTime + (this.suspended ? 0 : performance.now() - this.sampledAt));
    return this.lastTime;
  }

  stop(): Promise<void> {
    this.closing = true;
    this.stopping ??= this.commands.then(async () => {
      await invoke("stop_game", { sessionId: this.sessionId });
      if (this.failure !== undefined) throw this.failure;
    });
    return this.stopping;
  }
}
