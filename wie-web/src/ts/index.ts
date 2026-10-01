import { runApp } from "./app";
import type { Backend, LibraryApp } from "./backend";
import { initializeLibrary } from "./library";
import { initializeSettings } from "./settings";

const originalConsoleError = console.error;
console.error = (...args: unknown[]) => {
  window.alert(String(args[0]));
  originalConsoleError(...args);
};

const main = async (backend: Backend) => {
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

export const start = (backend: Backend) => {
  const initialize = () => {
    void main(backend).catch(error => {
      console.error(`라이브러리를 열 수 없습니다. ${String(error)}`, error);
    });
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialize);
  } else {
    initialize();
  }
};
