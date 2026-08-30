import { WorkletSynthesizer } from "spessasynth_lib";

type MidiEvent = [time: number, kind: "midi", data: Uint8Array];
type WaveEvent = [time: number, kind: "wave", channels: number, samplingRate: number, samples: Int16Array];
type TransportEvent = MidiEvent | WaveEvent;

type WorkerOutput =
  | { type: "event"; handle: number; deadline: number; event: TransportEvent }
  | { type: "cleanup"; handle: number; deadline: number; channels: number[]; notes: [number, number][]; immediate: boolean };

type AudioState = {
  synth: WorkletSynthesizer | null;
  ctx: AudioContext;
  clockAudioTime: number;
  clockPerformanceTime: number;
  midiGain: GainNode;
  pcmGain: GainNode;
  pcmSources: Map<number, Set<AudioBufferSourceNode>>;
};

let midiVolume = 0.5;
let pcmVolume = 0.5;

async function initAudio(): Promise<AudioState> {
  const ctx = new AudioContext();
  const midiGain = ctx.createGain();
  midiGain.gain.value = midiVolume;
  midiGain.connect(ctx.destination);

  const pcmGain = ctx.createGain();
  pcmGain.gain.value = pcmVolume;
  pcmGain.connect(ctx.destination);

  let synth: WorkletSynthesizer | null = null;
  try {
    await ctx.audioWorklet.addModule("/spessasynth_processor.min.js");
    synth = new WorkletSynthesizer(ctx);
    const buffer = await fetch("GeneralUser.sf3").then(response => response.arrayBuffer());
    await synth.soundBankManager.addSoundBank(buffer, "main");
    await synth.isReady;
    synth.connect(midiGain);
  } catch (error) {
    console.warn("MIDI output is unavailable:", error);
  }

  return {
    synth,
    ctx,
    clockAudioTime: ctx.currentTime,
    clockPerformanceTime: performance.timeOrigin + performance.now(),
    midiGain,
    pcmGain,
    pcmSources: new Map(),
  };
}

const audioReady: Promise<AudioState | null> = new Promise(resolve => {
  const start = () => initAudio().then(resolve, error => {
    console.warn("AudioPlayer init failed, audio will be silent:", error);
    resolve(null);
  });
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
  } else {
    start();
  }
});

export function setMasterVolume(value: number): void {
  midiVolume = value;
  audioReady.then(state => {
    if (state) state.midiGain.gain.value = value;
  });
}

export function setPcmVolume(value: number): void {
  pcmVolume = value;
  audioReady.then(state => {
    if (state) state.pcmGain.gain.value = value;
  });
}

export class AudioPlayer {
  private readonly worker = new Worker(new URL("./audio-worker.ts", import.meta.url), { type: "module" });
  private commands: Promise<void> = Promise.resolve();

  constructor() {
    this.worker.onmessage = (message: MessageEvent<WorkerOutput>) => {
      void audioReady.then(state => {
        if (!state) return;

        const output = message.data;
        const time = Math.max(state.ctx.currentTime, state.clockAudioTime + (output.deadline - state.clockPerformanceTime) / 1000);

        if (output.type === "event") {
          if (output.event[1] === "midi") {
            state.synth?.sendMessage(output.event[2], 0, { time });
            return;
          }

          const [, , channels, samplingRate, samples] = output.event;
          if (channels === 0 || samplingRate === 0) return;

          const frameCount = Math.floor(samples.length / channels);
          const buffer = state.ctx.createBuffer(channels, frameCount, samplingRate);
          for (let channel = 0; channel < channels; channel++) {
            const data = buffer.getChannelData(channel);
            for (let frame = 0; frame < frameCount; frame++) {
              data[frame] = samples[frame * channels + channel] / 32768;
            }
          }

          const source = state.ctx.createBufferSource();
          source.buffer = buffer;
          source.connect(state.pcmGain);
          const sources = state.pcmSources.get(output.handle) ?? new Set<AudioBufferSourceNode>();
          sources.add(source);
          state.pcmSources.set(output.handle, sources);
          source.onended = () => {
            sources.delete(source);
            if (sources.size === 0) state.pcmSources.delete(output.handle);
          };
          source.start(time);
          return;
        }

        const cleanupTimes = output.immediate && time > state.ctx.currentTime ? [state.ctx.currentTime, time] : [time];
        for (const cleanupTime of cleanupTimes) {
          for (const [channel, note] of output.notes) {
            state.synth?.sendMessage([0x80 | channel, note, 0], 0, { time: cleanupTime });
          }
          for (const channel of output.channels) {
            state.synth?.sendMessage([0xb0 | channel, 64, 0], 0, { time: cleanupTime });
            state.synth?.sendMessage([0xb0 | channel, 120, 0], 0, { time: cleanupTime });
            state.synth?.sendMessage([0xb0 | channel, 123, 0], 0, { time: cleanupTime });
          }
        }

        if (output.immediate) {
          for (const source of state.pcmSources.get(output.handle) ?? []) {
            source.stop(state.ctx.currentTime);
          }
        }
      });
    };
  }

  public play(handle: number, duration: number, events: TransportEvent[], repeat: boolean): void {
    const buffers = events.map(event => (event[1] === "midi" ? event[2].buffer : event[4].buffer));
    this.commands = this.commands
      .then(async () => {
        const state = await audioReady;
        if (state?.ctx.state === "suspended") {
          await state.ctx.resume();
          state.clockAudioTime = state.ctx.currentTime;
          state.clockPerformanceTime = performance.timeOrigin + performance.now();
        }
        this.worker.postMessage({ type: "play", handle, duration, events, repeat }, buffers);
      })
      .catch(error => console.warn("Failed to start audio playback:", error));
  }

  public stop(handle: number): void {
    this.commands = this.commands
      .then(async () => {
        await audioReady;
        this.worker.postMessage({ type: "stop", handle });
      })
      .catch(error => console.warn("Failed to stop audio playback:", error));
  }
}
