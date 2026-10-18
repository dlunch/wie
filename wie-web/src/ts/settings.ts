import { Languages, Music2, Volume2, X, createIcons } from "lucide";

import type { Settings, Wie } from "./backend";
import { getLanguagePreference, setLanguagePreference, t, type LanguagePreference } from "./i18n";

export interface SettingsController {
  get<K extends keyof Settings>(key: K): Settings[K];
  set<K extends keyof Settings>(key: K, value: Settings[K]): Promise<void>;
  open(): void;
}

export const initializeSettings = async (backend: Wie): Promise<SettingsController> => {
  const dialog = document.getElementById("settings-dialog") as HTMLDialogElement;
  const midiSlider = document.getElementById("volume-midi") as HTMLInputElement;
  const pcmSlider = document.getElementById("volume-pcm") as HTMLInputElement;
  const languageSelect = document.getElementById("language-preference") as HTMLSelectElement;
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
      languageSelect.value = getLanguagePreference();
      dialog.showModal();
    },
  };

  midiSlider.value = String(values.midiVolume * 100);
  pcmSlider.value = String(values.pcmVolume * 100);
  for (const [slider, key] of [[midiSlider, "midiVolume"], [pcmSlider, "pcmVolume"]] as const) {
    slider.addEventListener("input", () => {
      void settings.set(key, slider.valueAsNumber / 100).catch(error => {
        slider.value = String(values[key] * 100);
        window.alert(t("settingsSaveFailed", { error: String(error) }));
      });
    });
  }
  languageSelect.value = getLanguagePreference();
  languageSelect.addEventListener("change", () => {
    try {
      setLanguagePreference(languageSelect.value as LanguagePreference);
    } catch (error) {
      languageSelect.value = getLanguagePreference();
      window.alert(t("settingsSaveFailed", { error: String(error) }));
    }
  });
  createIcons({ icons: { Languages, Music2, Volume2, X }, root: dialog });

  return settings;
};
