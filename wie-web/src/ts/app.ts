import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Settings, createIcons } from "lucide";

import type { LibraryApp, PlayerSession, Wie } from "./backend";
import type { SettingsController } from "./settings";

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
  Enter: "OK",
  NumpadEnter: "OK",
  Space: "OK",
};
const icons = {
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  Settings,
};

export const runApp = (backend: Wie, app: LibraryApp, settings: SettingsController): Promise<void> => new Promise((resolve, reject) => {
  const playerView = document.getElementById("player-view") as HTMLElement;
  const playerTitle = document.getElementById("player-title") as HTMLElement;
  const playerStatus = document.getElementById("player-status") as HTMLElement;
  const playerWarning = document.getElementById("player-warning") as HTMLElement;
  const backToLibrary = document.getElementById("back-to-library") as HTMLButtonElement;
  const appSettings = document.getElementById("app-settings") as HTMLButtonElement;
  const buttons = playerView.querySelectorAll<HTMLButtonElement>("button[data-key]");

  const abortController = new AbortController();
  const inputs = new Map<string, string>();
  let session: PlayerSession | undefined;
  let preparing = true;
  let ending = false;
  let start: Promise<PlayerSession>;

  playerTitle.textContent = app.title;
  playerWarning.hidden = true;
  createIcons({ icons, root: playerView });

  const updateControls = () => {
    const busy = preparing || !session || ending;
    playerStatus.hidden = !preparing || ending;
    playerView.setAttribute("aria-busy", String(busy));
    for (const button of buttons) button.disabled = busy;
  };
  const finish = async (error?: unknown) => {
    if (ending) return;
    ending = true;
    abortController.abort();
    inputs.clear();
    updateControls();
    try {
      await (await start).stop();
      if (error !== undefined) reject(error);
      else resolve();
    } catch (stopError) {
      reject(error ?? stopError);
    } finally {
      playerStatus.hidden = true;
      playerWarning.hidden = true;
      playerView.removeAttribute("aria-busy");
    }
  };

  const setKey = (source: string, key?: string) => {
    if (!session || preparing || ending) return;
    const previous = inputs.get(source);
    if (previous === key) return;
    if (previous) {
      inputs.delete(source);
      if (![...inputs.values()].includes(previous)) void session.key(previous, false).catch(finish);
    }
    if (key) {
      const pressed = [...inputs.values()].includes(key);
      inputs.set(source, key);
      if (!pressed) void session.key(key, true).catch(finish);
    }
  };
  const releaseKeys = () => {
    inputs.clear();
    if (session && !ending) void session.releaseKeys().catch(finish);
  };

  backToLibrary.addEventListener("click", () => { void finish(); }, { signal: abortController.signal });
  appSettings.addEventListener("click", () => {
    releaseKeys();
    settings.open();
  }, { signal: abortController.signal });
  window.addEventListener("blur", releaseKeys, { signal: abortController.signal });
  window.addEventListener("pagehide", () => { void finish(); }, { signal: abortController.signal });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) releaseKeys();
  }, { signal: abortController.signal });

  for (const button of buttons) {
    const key = button.dataset.key!;
    button.addEventListener(
      "pointerdown",
      (event) => {
        event.preventDefault();
        button.setPointerCapture(event.pointerId);
        setKey(`pointer:${event.pointerId}`, key);
      },
      { signal: abortController.signal },
    );
    const releaseKey = (event: PointerEvent) => {
      event.preventDefault();
      setKey(`pointer:${event.pointerId}`);
    };
    button.addEventListener("pointerup", releaseKey, { signal: abortController.signal });
    button.addEventListener("pointercancel", releaseKey, { signal: abortController.signal });
    button.addEventListener("lostpointercapture", releaseKey, { signal: abortController.signal });
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
          setKey(`keyboard:${event.code}`, key);
        }
      }
    },
    { signal: abortController.signal },
  );
  document.addEventListener(
    "keyup",
    (event) => {
      const key = KEY_MAP[event.code];
      if (key) {
        event.preventDefault();
        setKey(`keyboard:${event.code}`);
      }
    },
    { signal: abortController.signal },
  );

  updateControls();
  start = backend.startGame(app.id, event => {
    if (ending) return;
    switch (event.type) {
      case "ready":
        preparing = false;
        updateControls();
        break;
      case "warning":
        playerWarning.textContent = event.message;
        playerWarning.hidden = false;
        break;
      case "stopped":
        void finish();
        break;
      case "error":
        void finish(new Error(event.message));
        break;
    }
  });
  void start.then(started => {
    session = started;
    updateControls();
  }).catch(finish);
});
