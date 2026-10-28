import { invoke, isTauri } from "@tauri-apps/api/core";
import * as Sentry from "@sentry/browser";
import { wasmIntegration } from "@sentry/wasm";

import { recordPageView } from "./analytics";
import { runApp } from "./app";
import { initializeWie, type LibraryApp } from "./backend";
import { getLocale, initializeLocalization, t } from "./i18n";
import { initializeLibrary } from "./library";
import { rustPanic } from "./panic";
import { initializeSettings } from "./settings";

const main = async () => {
  Sentry.init({
    dsn: process.env.SENTRY_DSN,
    environment: process.env.ENVIRONMENT,
    release: process.env.SENTRY_RELEASE,
    integrations: [wasmIntegration()],
  });
  const native = isTauri();
  const backend = await initializeWie();
  document.documentElement.classList.add(backend.rendering === "native" ? "native" : "browser");
  const output = document.getElementById("player-output") as HTMLDivElement;
  if (backend.rendering === "canvas") {
    output.className = "canvas-wrapper";
    const canvas = document.createElement("canvas");
    canvas.id = "canvas";
    canvas.width = 240;
    canvas.height = 320;
    output.prepend(canvas);
  } else {
    output.className = "native-preparation";
    const keypadTopRow = document.querySelector(".button-container p")!;
    keypadTopRow.prepend(document.getElementById("back-to-library")!);
    keypadTopRow.append(document.getElementById("app-settings")!);
  }

  if (native) {
    document.addEventListener("click", event => {
      const link = event.target instanceof Element ? event.target.closest<HTMLAnchorElement>("a[data-external-link]") : null;
      if (!link) return;
      event.preventDefault();
      void invoke("plugin:opener|open_url", { url: link.href }).catch(() => {});
    });
  }

  const libraryView = document.getElementById("library-view") as HTMLDivElement;
  const playerView = document.getElementById("player-view") as HTMLElement;
  const settings = await initializeSettings(backend);
  let pageLeaving = false;
  window.addEventListener("pagehide", () => { pageLeaving = true; });
  window.addEventListener("pageshow", () => {
    pageLeaving = false;
    if (!libraryView.hidden) recordPageView();
  });

  const routeToApp = async (app: LibraryApp) => {
    let failed = false;
    Sentry.setContext("emulated_app", { id: app.id, title: app.title, filename: app.filename });
    libraryView.hidden = true;
    playerView.hidden = false;
    try {
      failed = await runApp(backend, app, settings);
      if (failed) window.alert(t("restartRequired"));
    } catch (error) {
      failed = true;
      const failure = rustPanic ?? error;
      const panicked = rustPanic !== undefined;
      const showError = () => {
        window.alert(native ? t("restartRequired") : String(failure));
        if (panicked && !native) window.location.reload();
      };
      if (Sentry.isEnabled()) {
        const eventId = Sentry.captureException(failure, {
          mechanism: { type: "wie", handled: !panicked },
          captureContext: { tags: { error_kind: panicked ? "panic" : "handled" } },
        });
        Sentry.showReportDialog({
          eventId,
          lang: getLocale(),
          onError: showError,
          ...(native ? { subtitle: t("restartRequired") } : panicked ? {
            subtitle: t("emulatorStopped"),
            onClose: () => window.location.reload(),
          } : {}),
        });
      } else {
        showError();
      }
    } finally {
      Sentry.setContext("emulated_app", null);
      if (native && failed) {
        const warning = document.getElementById("player-warning") as HTMLElement;
        warning.textContent = t("restartRequired");
        warning.setAttribute("role", "alert");
        warning.hidden = false;
        playerView.querySelectorAll<HTMLButtonElement>("button").forEach(button => { button.disabled = true; });
      } else if (!rustPanic) {
        playerView.hidden = true;
        libraryView.hidden = false;
        if (!pageLeaving) recordPageView();
      }
    }
  };

  await initializeLibrary(backend, routeToApp, settings);

  const advertisingScripts = document.getElementById("advertising-scripts") as HTMLTemplateElement;
  try {
    const advertising = native
      ? await invoke<"adsense" | "adsense-test" | "sponsor">("initialize_advertising")
      : "adsense";
    if (advertising === "sponsor") {
      document.querySelector(".adsbygoogle")!.remove();
      document.querySelector<HTMLElement>(".library-sponsor")!.hidden = false;
    } else {
      document.querySelector(".library-sponsor")!.remove();
      if (advertising === "adsense-test" || process.env.NODE_ENV !== "production") {
        document.querySelector<HTMLElement>(".adsbygoogle")!.dataset.adtest = "on";
      }
      document.head.append(document.importNode(advertisingScripts.content, true));
    }
  } catch (error) {
    Sentry.captureException(error);
    console.error("Advertising initialization failed", error);
    document.querySelector(".library-ad")!.remove();
  } finally {
    advertisingScripts.remove();
  }
};

const start = () => {
  initializeLocalization();
  void main().catch(error => {
    Sentry.captureException(error);
    window.alert(isTauri() ? t("restartRequired") : t("libraryOpenFailed", { error: String(error) }));
    console.error(t("libraryOpenFailed", { error: String(error) }), error);
  });
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start);
} else {
  start();
}
