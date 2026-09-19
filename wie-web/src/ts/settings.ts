import { Cpu, Music2, Volume2, X, createIcons } from "lucide";

import { setMasterVolume } from "./midi";

const AOT_STORAGE_KEY = "wie_wasm_aot_enabled";

export interface SettingsController {
  readonly pcmVolume: number;
  readonly enableWasmAot: boolean;
  open(): void;
  onPcmVolumeChange(listener: (volume: number) => void): () => void;
}

export const initializeSettings = (): SettingsController => {
  const dialog = document.getElementById("settings-dialog") as HTMLDialogElement;
  const midiSlider = document.getElementById("volume-midi") as HTMLInputElement;
  const pcmSlider = document.getElementById("volume-pcm") as HTMLInputElement;
  const aotCheckbox = document.getElementById("enable-wasm-aot") as HTMLInputElement;

  aotCheckbox.checked = localStorage.getItem(AOT_STORAGE_KEY) === "true";
  aotCheckbox.addEventListener("change", () => localStorage.setItem(AOT_STORAGE_KEY, String(aotCheckbox.checked)));

  setMasterVolume(Number(midiSlider.value) / 100);
  midiSlider.addEventListener("input", () => setMasterVolume(Number(midiSlider.value) / 100));
  createIcons({ icons: { Cpu, Music2, Volume2, X }, root: dialog });

  return {
    get enableWasmAot() {
      return aotCheckbox.checked;
    },
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
