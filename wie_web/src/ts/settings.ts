import { Music2, Volume2, X, createIcons } from "lucide";

import { setMasterVolume } from "./midi";

export interface SettingsController {
  readonly pcmVolume: number;
  open(): void;
  onPcmVolumeChange(listener: (volume: number) => void): () => void;
}

export const initializeSettings = (): SettingsController => {
  const dialog = document.getElementById("settings-dialog") as HTMLDialogElement;
  const midiSlider = document.getElementById("volume-midi") as HTMLInputElement;
  const pcmSlider = document.getElementById("volume-pcm") as HTMLInputElement;

  setMasterVolume(Number(midiSlider.value) / 100);
  midiSlider.addEventListener("input", () => setMasterVolume(Number(midiSlider.value) / 100));
  createIcons({ icons: { Music2, Volume2, X }, root: dialog });

  return {
    get pcmVolume() {
      return Number(pcmSlider.value) / 100;
    },
    open() {
      dialog.showModal();
    },
    onPcmVolumeChange(listener) {
      const updateVolume = () => listener(Number(pcmSlider.value) / 100);
      pcmSlider.addEventListener("input", updateVolume);
      return () => pcmSlider.removeEventListener("input", updateVolume);
    },
  };
};
