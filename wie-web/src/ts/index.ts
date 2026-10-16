import { isTauri } from "@tauri-apps/api/core";
import * as Sentry from "@sentry/browser";
import { wasmIntegration } from "@sentry/wasm";

import { recordPageView } from "./analytics";
import { runApp } from "./app";
import { initializeWie, type LibraryApp } from "./backend";
import { initializeLibrary } from "./library";
import { rustPanic } from "./panic";
import { initializeSettings } from "./settings";

const RESTART_MESSAGE = "오류로 실행이 중단되었습니다. wie 앱을 완전히 종료한 뒤 다시 실행해 주세요.";

const main = async () => {
  Sentry.init({
    dsn: "https://218d5a11f3e753fe4b60f92e5c1e64b0@o106536.ingest.us.sentry.io/4512049004412928",
    environment: process.env.ENVIRONMENT,
    release: process.env.SENTRY_RELEASE,
    integrations: [wasmIntegration()],
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
  let pageLeaving = false;
  if (!native) {
    window.addEventListener("pagehide", () => { pageLeaving = true; });
    window.addEventListener("pageshow", () => {
      pageLeaving = false;
      if (!libraryView.hidden) recordPageView();
    });
  }

  const routeToApp = async (app: LibraryApp) => {
    let failed = false;
    Sentry.setContext("emulated_app", { id: app.id, title: app.title, filename: app.filename });
    libraryView.hidden = true;
    playerView.hidden = false;
    try {
      failed = await runApp(backend, app, settings, !native);
      if (failed) window.alert(RESTART_MESSAGE);
    } catch (error) {
      failed = true;
      const failure = rustPanic ?? error;
      const panicked = rustPanic !== undefined;
      const eventId = Sentry.captureException(failure, {
        mechanism: { type: "wie", handled: !panicked },
        captureContext: { tags: { error_kind: panicked ? "panic" : "handled" } },
      });
      Sentry.showReportDialog({
        eventId,
        onError: () => {
          window.alert(native ? RESTART_MESSAGE : String(failure));
          if (panicked && !native) window.location.reload();
        },
        ...(native ? { subtitle: RESTART_MESSAGE } : panicked ? {
          subtitle: "에뮬레이터가 중단되었습니다. 다시 실행하려면 페이지를 새로고침해 주세요.",
          onClose: () => window.location.reload(),
        } : {}),
      });
    } finally {
      Sentry.setContext("emulated_app", null);
      if (native && failed) {
        const warning = document.getElementById("player-warning") as HTMLElement;
        warning.textContent = RESTART_MESSAGE;
        warning.setAttribute("role", "alert");
        warning.hidden = false;
        playerView.querySelectorAll<HTMLButtonElement>("button").forEach(button => { button.disabled = true; });
      } else if (!rustPanic) {
        playerView.hidden = true;
        libraryView.hidden = false;
        if (!native && !pageLeaving) recordPageView();
      }
    }
  };

  await initializeLibrary(backend, routeToApp, settings);
};

const start = () => {
  void main().catch(error => {
    Sentry.captureException(error);
    window.alert(isTauri() ? RESTART_MESSAGE : `라이브러리를 열 수 없습니다. ${String(error)}`);
    console.error(`라이브러리를 열 수 없습니다. ${String(error)}`, error);
  });
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start);
} else {
  start();
}
