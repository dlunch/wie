type MidiEvent = [time: number, kind: "midi", data: Uint8Array];
type WaveEvent = [time: number, kind: "wave", channels: number, samplingRate: number, samples: Int16Array];
type TransportEvent = MidiEvent | WaveEvent;

type WorkerCommand =
  | { type: "play"; handle: number; duration: number; events: TransportEvent[]; repeat: boolean }
  | { type: "stop"; handle: number };

type WorkerOutput =
  | { type: "event"; handle: number; deadline: number; event: TransportEvent }
  | { type: "cleanup"; handle: number; deadline: number; channels: number[]; notes: [number, number][]; immediate: boolean };

type Playback = {
  duration: number;
  events: TransportEvent[];
  repeat: boolean;
  startedAt: number;
  nextEvent: number;
  lastScheduledAt: number;
  cleanupAt: number | null;
  activeNotes: Set<number>;
  usedChannels: Set<number>;
};

const LOOKAHEAD_MS = 50;
const playbacks = new Map<number, Playback>();
const blockedUntil = new Map<number, number>();
let timer: ReturnType<typeof setTimeout> | undefined;

const scope = self as unknown as {
  onmessage: ((message: MessageEvent<WorkerCommand>) => void) | null;
  postMessage(message: WorkerOutput): void;
};

scope.onmessage = message => {
  const command = message.data;
  if (command.type === "play") {
    const oldPlayback = playbacks.get(command.handle);
    if (oldPlayback) stopPlayback(command.handle, oldPlayback);

    const now = performance.now();
    const startedAt = Math.max(now, blockedUntil.get(command.handle) ?? now);
    blockedUntil.delete(command.handle);
    playbacks.set(command.handle, {
      duration: command.duration,
      events: command.events,
      repeat: command.repeat,
      startedAt,
      nextEvent: 0,
      lastScheduledAt: startedAt,
      cleanupAt: null,
      activeNotes: new Set(),
      usedChannels: new Set(),
    });
  } else {
    const playback = playbacks.get(command.handle);
    if (playback) stopPlayback(command.handle, playback);
  }

  schedule();
};

function schedule(): void {
  if (timer !== undefined) {
    clearTimeout(timer);
    timer = undefined;
  }

  const now = performance.now();
  const horizon = now + LOOKAHEAD_MS;

  for (const [handle, deadline] of blockedUntil) {
    if (deadline <= now) blockedUntil.delete(handle);
  }

  for (const [handle, playback] of playbacks) {
    if (playback.cleanupAt !== null) {
      if (playback.cleanupAt <= now) playbacks.delete(handle);
      continue;
    }

    while (true) {
      const event = playback.events[playback.nextEvent];
      if (event) {
        const deadline = playback.startedAt + event[0];
        if (deadline > horizon) break;

        trackMidi(playback, event);
        playback.lastScheduledAt = Math.max(playback.lastScheduledAt, deadline);
        playback.nextEvent++;
        scope.postMessage({ type: "event", handle, deadline: performance.timeOrigin + deadline, event });
        continue;
      }

      const end = playback.startedAt + playback.duration;
      if (end > horizon) break;

      playback.lastScheduledAt = Math.max(playback.lastScheduledAt, end);
      postCleanup(handle, playback, end, false);
      if (!playback.repeat || playback.duration === 0) {
        if (end <= now) {
          playbacks.delete(handle);
        } else {
          playback.cleanupAt = end;
        }
        break;
      }

      playback.activeNotes.clear();
      playback.usedChannels.clear();
      playback.startedAt = Math.max(end, now);
      playback.nextEvent = 0;
      playback.lastScheduledAt = playback.startedAt;
    }
  }

  let nextDelay = Number.POSITIVE_INFINITY;
  for (const playback of playbacks.values()) {
    if (playback.cleanupAt !== null) {
      nextDelay = Math.min(nextDelay, playback.cleanupAt - now);
    } else {
      const event = playback.events[playback.nextEvent];
      const deadline = event ? playback.startedAt + event[0] : playback.startedAt + playback.duration;
      nextDelay = Math.min(nextDelay, deadline - now - LOOKAHEAD_MS);
    }
  }
  for (const deadline of blockedUntil.values()) {
    nextDelay = Math.min(nextDelay, deadline - now);
  }
  if (nextDelay !== Number.POSITIVE_INFINITY) {
    timer = setTimeout(schedule, Math.max(0, nextDelay));
  }
}

function stopPlayback(handle: number, playback: Playback): void {
  playbacks.delete(handle);
  const deadline = Math.max(performance.now(), playback.lastScheduledAt) + 1;
  blockedUntil.set(handle, deadline);
  postCleanup(handle, playback, deadline, true);
}

function trackMidi(playback: Playback, event: TransportEvent): void {
  if (event[1] !== "midi" || event[2].length === 0) return;

  const status = event[2][0];
  if (status < 0x80 || status >= 0xf0) return;

  const channel = status & 0x0f;
  playback.usedChannels.add(channel);
  if (event[2].length < 2) return;

  const noteKey = (channel << 8) | event[2][1];
  if ((status & 0xf0) === 0x80 || ((status & 0xf0) === 0x90 && (event[2][2] ?? 0) === 0)) {
    playback.activeNotes.delete(noteKey);
  } else if ((status & 0xf0) === 0x90) {
    playback.activeNotes.add(noteKey);
  }
}

function postCleanup(handle: number, playback: Playback, deadline: number, immediate: boolean): void {
  const notes = [...playback.activeNotes].map(note => [note >> 8, note & 0xff] as [number, number]);
  scope.postMessage({
    type: "cleanup",
    handle,
    deadline: performance.timeOrigin + deadline,
    channels: [...playback.usedChannels],
    notes,
    immediate,
  });
}
