import * as Sentry from "@sentry/browser";

import { runApp } from "./app";
import { AppMetadata } from "./app_library_store";
import { initializeLibrary } from "./library";
import { initializeSettings } from "./settings";

Sentry.init({
  dsn: "https://fa9187d6bd7dd43ae621f26d33641f81@o106536.ingest.us.sentry.io/4512048969678848",
});

const originalConsoleError = console.error;
console.error = (...args: unknown[]) => {
  window.alert(String(args[0]));
  originalConsoleError(...args);
};

const main = async () => {
  const libraryView = document.getElementById("library-view") as HTMLDivElement;
  const playerView = document.getElementById("player-view") as HTMLElement;
  const settings = initializeSettings();
  const fontResponse = await fetch(
    new URL("../../../assets/neodgm.ttf", import.meta.url),
  );
  if (!fontResponse.ok) {
    throw new Error(
      `Failed to load font: ${fontResponse.status} ${fontResponse.statusText}`,
    );
  }
  const fontData = new Uint8Array(await fontResponse.arrayBuffer());

  const routeToApp = (app: AppMetadata, archive: Uint8Array) =>
    new Promise<void>((resolve, reject) => {
      libraryView.hidden = true;
      playerView.hidden = false;

      let disposeApp: () => void;
      const routeToLibrary = (error?: unknown) => {
        disposeApp();
        playerView.hidden = true;
        libraryView.hidden = false;

        if (error !== undefined) {
          reject(error);
        } else {
          resolve();
        }
      };

      try {
        disposeApp = runApp(app, archive, fontData, settings, routeToLibrary);
      } catch (error) {
        playerView.hidden = true;
        libraryView.hidden = false;
        reject(error);
      }
    });

  await initializeLibrary(routeToApp, settings);
};

const start = () => {
  void main().catch((error) => {
    console.error(`라이브러리를 열 수 없습니다. ${String(error)}`, error);
  });
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start);
} else {
  start();
}
