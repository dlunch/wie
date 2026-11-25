import { onBackButtonPress } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";

// Lets the user edit `initial`. Resolves with the result, or null when the user dismisses the dialog.
export const requestTextInput = (initial: string): Promise<string | null> => new Promise(resolve => {
  const dialog = document.getElementById("text-input-dialog") as HTMLDialogElement;
  const input = document.getElementById("text-input") as HTMLInputElement;
  const viewport = window.visualViewport!;
  const closed = new AbortController();
  // The on-screen keyboard shrinks only the visual viewport, so the docked bar is lifted above it.
  const dock = () => dialog.style.setProperty("--keyboard-inset", `${window.innerHeight - viewport.height - viewport.offsetTop}px`);

  // Android's back button would otherwise leave the app instead of dismissing the dialog.
  const backButton = isTauri() ? onBackButtonPress(() => dialog.close()) : undefined;

  input.value = initial;
  dialog.returnValue = "";
  viewport.addEventListener("resize", dock, { signal: closed.signal });
  viewport.addEventListener("scroll", dock, { signal: closed.signal });
  dialog.addEventListener("close", () => {
    closed.abort();
    void backButton?.then(listener => listener.unregister());
    resolve(dialog.returnValue === "confirm" ? input.value : null);
  }, { once: true });
  dock();
  dialog.showModal();
});
