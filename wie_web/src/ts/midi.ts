import { WorkletSynthesizer } from 'spessasynth_lib';

type AudioState = { synth: WorkletSynthesizer; ctx: AudioContext; gain: GainNode };

let masterVolume = 0.5;

async function initAudio(): Promise<AudioState> {
  const ctx = new AudioContext();
  await ctx.audioWorklet.addModule('/spessasynth_processor.min.js');
  const synth = new WorkletSynthesizer(ctx);
  const buffer = await fetch('GeneralUser.sf3').then(r => r.arrayBuffer());
  await synth.soundBankManager.addSoundBank(buffer, 'main');
  await synth.isReady;
  const gain = ctx.createGain();
  gain.gain.value = masterVolume;
  synth.connect(gain);
  gain.connect(ctx.destination);
  return { synth, ctx, gain };
}

// Starts loading immediately on DOMContentLoaded.
// ctx.resume() is called lazily on first note_on (requires user gesture).
const audioReady: Promise<AudioState | null> = new Promise(resolve => {
  const start = () => initAudio().then(resolve, err => {
    console.warn('MidiPlayer init failed, audio will be silent:', err);
    resolve(null);
  });
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start, { once: true });
  } else {
    start();
  }
});

export function setMasterVolume(value: number): void {
  masterVolume = value;
  audioReady.then(s => {
    if (s) s.gain.gain.value = value;
  });
}

export class MidiPlayer {
  constructor() {}

  public note_on(channel_id: number, note: number, velocity: number): void {
    if (velocity === 0) { this.note_off(channel_id, note, 0); return; }
    audioReady.then(async s => {
      if (!s) return;
      if (s.ctx.state === 'suspended') await s.ctx.resume();
      s.synth.noteOn(channel_id, note, velocity);
    });
  }

  public note_off(channel_id: number, note: number, _velocity: number): void {
    audioReady.then(s => s?.synth.noteOff(channel_id, note));
  }

  public control_change(channel_id: number, control: number, value: number): void {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    audioReady.then(s => s?.synth.controllerChange(channel_id, control as any, value));
  }

  public program_change(channel_id: number, program: number): void {
    audioReady.then(s => s?.synth.programChange(channel_id, program));
  }
}
