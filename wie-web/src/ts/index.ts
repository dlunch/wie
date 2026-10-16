import { isTauri } from "@tauri-apps/api/core";
import * as Sentry from "@sentry/browser";

import { runApp } from "./app";
import { initializeWie, type LibraryApp } from "./backend";
import { initializeLibrary } from "./library";
import { rustPanic } from "./panic";
import { initializeSettings } from "./settings";

const main = async () => {
  Sentry.init({
    dsn: "https://218d5a11f3e753fe4b60f92e5c1e64b0@o106536.ingest.us.sentry.io/4512049004412928",
    // Each failed run needs an event ID that the feedback dialog can reference.
    integrations: defaults => defaults.filter(integration => integration.name !== "Dedupe"),
  });
  const native = isTauri();
  const backend = await initializeWie();
  document.documentElement.classList.add(backend.rendering === "native" ? "native" : "browser");
  const output = document.getElementById("player-output") as HTMLDivElement;
  const browserScripts = document.getElementById("browser-scripts") as HTMLTemplateElement;
  if (native) {
    document.querySelector(".library-ad")!.remove();
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
    Sentry.setContext("emulated_app", { id: app.id, title: app.title, filename: app.filename });
    libraryView.hidden = true;
    playerView.hidden = false;
    try {
      await runApp(backend, app, settings);
    } catch (error) {
      const failure = rustPanic ?? error;
      const panicked = rustPanic !== undefined;
      const eventId = Sentry.captureException(failure, {
        mechanism: { type: "wie", handled: !panicked },
        captureContext: { tags: { error_kind: panicked ? "panic" : "handled" } },
      });
      Sentry.showReportDialog({
        eventId,
        onError: () => {
          window.alert(String(failure));
          if (panicked) window.location.reload();
        },
        ...(panicked ? {
          subtitle: "에뮬레이터가 중단되었습니다. 다시 실행하려면 페이지를 새로고침해 주세요.",
          onClose: () => window.location.reload(),
        } : {}),
      });
    } finally {
      Sentry.setContext("emulated_app", null);
      if (!rustPanic) {
        playerView.hidden = true;
        libraryView.hidden = false;
      }
    }
  };

  await initializeLibrary(backend, routeToApp, settings);
};

const start = () => {
  void main().catch(error => {
    Sentry.captureException(error);
    window.alert(`라이브러리를 열 수 없습니다. ${String(error)}`);
    console.error(`라이브러리를 열 수 없습니다. ${String(error)}`, error);
  });
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start);
} else {
  start();
}
