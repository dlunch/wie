import { WieWeb } from "@pkg";
import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Settings, createIcons } from "lucide";

import { AppMetadata } from "./app_library_store";
import { SettingsController } from "./settings";

const KEY_MAP: Record<string, string> = {
  Digit1: "1",
  Digit2: "2",
  Digit3: "3",
  KeyQ: "4",
  KeyW: "5",
  KeyE: "6",
  KeyA: "7",
  KeyS: "8",
  KeyD: "9",
  KeyZ: "*",
  KeyX: "0",
  KeyC: "#",
  Backspace: "CLR",
  ArrowUp: "UP",
  ArrowLeft: "LEFT",
  ArrowRight: "RIGHT",
  ArrowDown: "DOWN",
  Space: "OK",
};
const icons = {
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  Settings,
};

export const runApp = (app: AppMetadata, archive: Uint8Array, fontData: Uint8Array, settings: SettingsController, exit: (error?: unknown) => void) => {
  const playerView = document.getElementById("player-view") as HTMLElement;
  const playerTitle = document.getElementById("player-title") as HTMLElement;
  const canvas = document.getElementById("canvas") as HTMLCanvasElement;
  const backToLibrary = document.getElementById("back-to-library") as HTMLButtonElement;
  const appSettings = document.getElementById("app-settings") as HTMLButtonElement;

  const abortController = new AbortController();
  const wieWeb = new WieWeb(app.filename, archive, canvas, fontData);
  const unsubscribePcmVolume = settings.onPcmVolumeChange((volume) => wieWeb.set_pcm_volume(volume));
  let running = true;

  wieWeb.set_pcm_volume(settings.pcmVolume);
  playerTitle.textContent = app.title;
  createIcons({ icons, root: playerView });

  backToLibrary.addEventListener("click", () => exit(), { signal: abortController.signal });
  appSettings.addEventListener("click", settings.open, { signal: abortController.signal });

  for (const button of document.querySelectorAll<HTMLButtonElement>("button[data-key]")) {
    const key = button.dataset.key!;
    button.addEventListener(
      "pointerdown",
      (event) => {
        event.preventDefault();
        button.setPointerCapture(event.pointerId);
        wieWeb.key_down(key);
      },
      { signal: abortController.signal },
    );
    const releaseKey = (event: PointerEvent) => {
      event.preventDefault();
      wieWeb.key_up(key);
    };
    button.addEventListener("pointerup", releaseKey, { signal: abortController.signal });
    button.addEventListener("pointercancel", releaseKey, { signal: abortController.signal });
  }

  document.addEventListener(
    "keydown",
    (event) => {
      if (event.target instanceof HTMLElement && event.target.closest("dialog")) {
        return;
      }

      const key = KEY_MAP[event.code];
      if (key) {
        event.preventDefault();
        if (!event.repeat) {
          wieWeb.key_down(key);
        }
      }
    },
    { signal: abortController.signal },
  );
  document.addEventListener(
    "keyup",
    (event) => {
      if (event.target instanceof HTMLElement && event.target.closest("dialog")) {
        return;
      }

      const key = KEY_MAP[event.code];
      if (key) {
        event.preventDefault();
        wieWeb.key_up(key);
      }
    },
    { signal: abortController.signal },
  );

  const update = () => {
    if (!running) {
      return;
    }

    try {
      wieWeb.update();
      requestAnimationFrame(update);
    } catch (error) {
      exit(error);
    }
  };
  requestAnimationFrame(update);

  return () => {
    running = false;
    abortController.abort();
    unsubscribePcmVolume();
    wieWeb.free();
  };
};
