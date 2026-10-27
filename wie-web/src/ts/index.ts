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
    dsn: "https://218d5a11f3e753fe4b60f92e5c1e64b0@o106536.ingest.us.sentry.io/4512049004412928",
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
  const advertising = native
    ? await invoke<"adsense" | "adsense-test" | "sponsor">("initialize_advertising")
    : "adsense";
  const advertisingScripts = document.getElementById("advertising-scripts") as HTMLTemplateElement;
  if (advertising === "sponsor") {
    const area = document.querySelector<HTMLElement>(".library-ad")!;
    area.dataset.i18nAriaLabel = "sponsorship";
    area.setAttribute("aria-label", t("sponsorship"));
    const link = document.createElement("a");
    link.className = "library-sponsor";
    link.href = "https://github.com/sponsors/dlunch";
    link.target = "_blank";
    link.rel = "noopener noreferrer";
    link.dataset.externalLink = "";
    link.dataset.i18nText = "supportDevelopment";
    link.textContent = t("supportDevelopment");
    area.replaceChildren(link);
  } else {
    if (advertising === "adsense-test" || process.env.NODE_ENV !== "production") {
      document.querySelector<HTMLElement>(".adsbygoogle")!.dataset.adtest = "on";
    }
    document.head.append(document.importNode(advertisingScripts.content, true));
  }
  advertisingScripts.remove();

  const analyticsScripts = document.getElementById("analytics-scripts") as HTMLTemplateElement;
  if (!native) {
    document.head.append(document.importNode(analyticsScripts.content, true));
  } else {
    for (const link of document.querySelectorAll<HTMLAnchorElement>("a[data-external-link]")) {
      link.addEventListener("click", event => {
        event.preventDefault();
        void invoke("plugin:opener|open_url", { url: link.href }).catch(error => {
          window.alert(t("linkOpenFailed", { error: String(error) }));
        });
      });
    }
  }
  analyticsScripts.remove();

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
    if (backend.rendering === "canvas" && window.matchMedia("(max-width: 600px)").matches) {
      requestAnimationFrame(() => window.scrollTo({ top: document.documentElement.scrollHeight, behavior: "instant" }));
    }
    try {
      failed = await runApp(backend, app, settings, !native);
      if (failed) window.alert(t("restartRequired"));
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
        lang: getLocale(),
        onError: () => {
          window.alert(native ? t("restartRequired") : String(failure));
          if (panicked && !native) window.location.reload();
        },
        ...(native ? { subtitle: t("restartRequired") } : panicked ? {
          subtitle: t("emulatorStopped"),
          onClose: () => window.location.reload(),
        } : {}),
      });
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
        if (!native && !pageLeaving) recordPageView();
      }
    }
  };

  await initializeLibrary(backend, routeToApp, settings);
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
