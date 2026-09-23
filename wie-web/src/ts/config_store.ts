const defaults = {
  midiVolume: 0.5,
  pcmVolume: 0.5,
  enableWasmAot: false,
  helpDismissed: false,
  welcomeSeen: false,
};

type Config = typeof defaults;

export const configStore = {
  get<K extends keyof Config>(key: K): Config[K] {
    try {
      const value: unknown = JSON.parse(localStorage.getItem(`wie_config_${key}`) ?? "null");
      if (typeof value === typeof defaults[key] && (typeof value !== "number" || (value >= 0 && value <= 1))) {
        return value as Config[K];
      }
    } catch {
      // Missing access or malformed stored values use the setting's default.
    }
    return defaults[key];
  },
  set<K extends keyof Config>(key: K, value: Config[K]): void {
    localStorage.setItem(`wie_config_${key}`, JSON.stringify(value));
  },
};
