import { Cpu, Music2, Volume2, X, createIcons } from "lucide";

import { configStore } from "./config_store";
import { setMasterVolume } from "./midi";

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

  aotCheckbox.checked = configStore.get("enableWasmAot");
  aotCheckbox.addEventListener("change", () => configStore.set("enableWasmAot", aotCheckbox.checked));

  midiSlider.value = String(configStore.get("midiVolume") * 100);
  pcmSlider.value = String(configStore.get("pcmVolume") * 100);
  setMasterVolume(Number(midiSlider.value) / 100);
  midiSlider.addEventListener("input", () => {
    const volume = Number(midiSlider.value) / 100;
    setMasterVolume(volume);
    configStore.set("midiVolume", volume);
  });
  pcmSlider.addEventListener("input", () => configStore.set("pcmVolume", Number(pcmSlider.value) / 100));
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
