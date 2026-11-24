// Resolves with the entered text, or null when the user dismisses the dialog.
export const requestTextInput = (): Promise<string | null> => new Promise(resolve => {
  const dialog = document.getElementById("text-input-dialog") as HTMLDialogElement;
  const input = document.getElementById("text-input") as HTMLInputElement;
  const viewport = window.visualViewport!;
  const closed = new AbortController();
  // The on-screen keyboard shrinks only the visual viewport, so the docked bar is lifted above it.
  const dock = () => dialog.style.setProperty("--keyboard-inset", `${window.innerHeight - viewport.height - viewport.offsetTop}px`);

  input.value = "";
  dialog.returnValue = "";
  viewport.addEventListener("resize", dock, { signal: closed.signal });
  viewport.addEventListener("scroll", dock, { signal: closed.signal });
  dialog.addEventListener("close", () => {
    closed.abort();
    resolve(dialog.returnValue === "confirm" ? input.value : null);
  }, { once: true });
  dock();
  dialog.showModal();
});
