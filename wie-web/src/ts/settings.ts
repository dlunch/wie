import { Cpu, Music2, Volume2, X, createIcons } from "lucide";

import type { Settings, Wie } from "./backend";

export interface SettingsController {
  get<K extends keyof Settings>(key: K): Settings[K];
  set<K extends keyof Settings>(key: K, value: Settings[K]): Promise<void>;
  open(): void;
}

export const initializeSettings = async (backend: Wie): Promise<SettingsController> => {
  const dialog = document.getElementById("settings-dialog") as HTMLDialogElement;
  const midiSlider = document.getElementById("volume-midi") as HTMLInputElement;
  const pcmSlider = document.getElementById("volume-pcm") as HTMLInputElement;
  const aotCheckbox = document.getElementById("enable-wasm-aot") as HTMLInputElement | null;
  let values = await backend.readSettings();
  let pending = Promise.resolve();

  const settings: SettingsController = {
    get(key) {
      return values[key];
    },
    set(key, value) {
      const saved = pending.then(async () => {
        const next = { ...values, [key]: value };
        await backend.writeSettings(next);
        values = next;
      });
      pending = saved.catch(() => {});
      return saved;
    },
    open() {
      dialog.showModal();
    },
  };

  midiSlider.value = String(values.midiVolume * 100);
  pcmSlider.value = String(values.pcmVolume * 100);
  for (const [slider, key] of [[midiSlider, "midiVolume"], [pcmSlider, "pcmVolume"]] as const) {
    slider.addEventListener("input", () => {
      void settings.set(key, slider.valueAsNumber / 100).catch(error => {
        slider.value = String(values[key] * 100);
        window.alert(`설정을 저장할 수 없습니다. ${String(error)}`);
      });
    });
  }
  if (aotCheckbox) {
    aotCheckbox.checked = values.enableWasmAot ?? false;
    aotCheckbox.addEventListener("change", () => {
      void settings.set("enableWasmAot", aotCheckbox.checked).catch(error => {
        aotCheckbox.checked = values.enableWasmAot ?? false;
        window.alert(`설정을 저장할 수 없습니다. ${String(error)}`);
      });
    });
  }
  createIcons({ icons: { Cpu, Music2, Volume2, X }, root: dialog });

  return settings;
};
