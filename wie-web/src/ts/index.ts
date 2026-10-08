import { isTauri } from "@tauri-apps/api/core";

import { runApp } from "./app";
import { initializeWie, type LibraryApp } from "./backend";
import { initializeLibrary } from "./library";
import { initializeSettings } from "./settings";

const originalConsoleError = console.error;
console.error = (...args: unknown[]) => {
  window.alert(String(args[0]));
  originalConsoleError(...args);
};

const main = async () => {
  const native = isTauri();
  const backend = await initializeWie();
  document.documentElement.classList.add(backend.rendering === "native" ? "native" : "browser");
  const output = document.getElementById("player-output") as HTMLDivElement;
  const browserScripts = document.getElementById("browser-scripts") as HTMLTemplateElement;
  if (native) {
    document.querySelector(".library-ad")!.remove();
    document.getElementById("enable-wasm-aot")!.closest("label")!.remove();
  }
  if (backend.rendering === "canvas") {
    output.className = "canvas-wrapper";
    const canvas = document.createElement("canvas");
    canvas.id = "canvas";
    canvas.width = 240;
    canvas.height = 320;
    output.prepend(canvas);
  } else {
    output.className = "native-preparation";
  }
  if (!native) document.head.append(document.importNode(browserScripts.content, true));
  browserScripts.remove();

  const libraryView = document.getElementById("library-view") as HTMLDivElement;
  const playerView = document.getElementById("player-view") as HTMLElement;
  const settings = await initializeSettings(backend);

  const routeToApp = async (app: LibraryApp) => {
    libraryView.hidden = true;
    playerView.hidden = false;
    try {
      await runApp(backend, app, settings);
    } finally {
      playerView.hidden = true;
      libraryView.hidden = false;
    }
  };

  await initializeLibrary(backend, routeToApp, settings);
};

const start = () => {
  void main().catch(error => {
    console.error(`라이브러리를 열 수 없습니다. ${String(error)}`, error);
  });
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start);
} else {
  start();
}
