import { WieWeb } from "@pkg";
import { setMasterVolume } from "./midi";

const TUTORIAL_STORAGE_KEY = "wie_tutorial_dismissed";

const initTutorial = () => {
  const overlay = document.getElementById("tutorial-overlay");
  const closeButton = document.getElementById("close-tutorial");
  const dontShowCheckbox = document.getElementById("dont-show-again") as HTMLInputElement;

  if (!overlay || !closeButton || !dontShowCheckbox) return;

  if (localStorage.getItem(TUTORIAL_STORAGE_KEY) !== "true") {
    overlay.classList.add("visible");
  }

  const closeTutorial = () => {
    if (dontShowCheckbox.checked) {
      localStorage.setItem(TUTORIAL_STORAGE_KEY, "true");
    } else {
      localStorage.removeItem(TUTORIAL_STORAGE_KEY);
    }
    overlay.classList.remove("visible");
  };

  closeButton.addEventListener("click", closeTutorial);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) {
      closeTutorial();
    }
  });

  const showTutorialButton = document.getElementById("show-tutorial");
  showTutorialButton?.addEventListener("click", () => {
    overlay.classList.add("visible");
  });
};

const key_map: Record<string, string> = {
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
  Space: "OK",
};

const main = () => {
  initVolume();

  const file = document.getElementById("file") as HTMLInputElement;
  const fileName = document.getElementById("file-name");

  file.addEventListener("change", () => {
    const selected_file = file.files?.[0];
    if (fileName) {
      fileName.textContent = selected_file?.name ?? "파일 선택";
    }

    if (selected_file) {
      const prestart = document.getElementById("prestart");
      const cornerActions = document.querySelector(".corner-actions");
      const settingsToggle = document.getElementById("settings-toggle");
      const settingsPanel = document.getElementById("settings-panel");
      const buttonContainer = document.querySelector(".button-container");
      if (prestart) {
        prestart.classList.add("hidden");
        cornerActions?.classList.add("hidden");
        setTimeout(() => {
          prestart.remove();
          if (settingsToggle && settingsPanel && buttonContainer) {
            buttonContainer.appendChild(settingsToggle);
            buttonContainer.appendChild(settingsPanel);
          }
          cornerActions?.remove();
        }, 300);
      }

      const original_console_error = console.error;
      console.error = (message: string, ...args) => {
        alert(message);
        original_console_error(message, ...args);
      };

      const reader = new FileReader();

      reader.onload = (e) => {
        const data = e.target?.result as ArrayBuffer;

        try {
          const canvas = document.getElementById("canvas") as HTMLCanvasElement;
          const wie_web = new WieWeb(
            selected_file.name,
            new Uint8Array(data),
            canvas
          );
          const pcmSlider = document.getElementById("volume-pcm") as HTMLInputElement;
          wie_web.set_pcm_volume(Number(pcmSlider.value) / 100);
          pcmSlider?.addEventListener("input", () => wie_web.set_pcm_volume(Number(pcmSlider.value) / 100));

          for (const button of document.querySelectorAll("button[data-key]")) {
            const handleKeyDown = (e: Event) => {
              e.preventDefault();
              const key = (e.currentTarget as HTMLButtonElement).dataset.key;
              if (key) wie_web.key_down(key);
            };
            const handleKeyUp = (e: Event) => {
              e.preventDefault();
              const key = (e.currentTarget as HTMLButtonElement).dataset.key;
              if (key) wie_web.key_up(key);
            };

            button.addEventListener("mousedown", handleKeyDown);
            button.addEventListener("mouseup", handleKeyUp);
            button.addEventListener("touchstart", handleKeyDown);
            button.addEventListener("touchend", handleKeyUp);
          }
          document.addEventListener("keydown", (e) => {
            if (key_map[e.code]) {
              e.preventDefault();
              wie_web.key_down(key_map[e.code]);
            }
          });
          document.addEventListener("keyup", (e) => {
            if (key_map[e.code]) {
              e.preventDefault();
              wie_web.key_up(key_map[e.code]);
            }
          });
          const update = () => {
            try {
              wie_web.update();
            } catch (e) {
              wie_web.free();
              alert(String(e));
              throw e;
            }

            requestAnimationFrame(update);
          };

          requestAnimationFrame(update);
        } catch (e) {
          alert(String(e));
          throw e;
        }
      };

      reader.readAsArrayBuffer(selected_file);
    }
  });
};

const initVolume = () => {
  const toggle = document.getElementById("settings-toggle");
  const panel = document.getElementById("settings-panel");
  const midiSlider = document.getElementById("volume-midi") as HTMLInputElement;

  toggle?.addEventListener("click", () => panel?.classList.toggle("visible"));
  document.addEventListener("click", (e) => {
    if (!toggle?.contains(e.target as Node) && !panel?.contains(e.target as Node)) {
      panel?.classList.remove("visible");
    }
  });

  setMasterVolume(Number(midiSlider.value) / 100);
  midiSlider?.addEventListener("input", () => setMasterVolume(Number(midiSlider.value) / 100));
};

if (document.readyState !== "loading") {
  initTutorial();
  main();
} else {
  document.addEventListener("DOMContentLoaded", () => {
    initTutorial();
    main();
  });
}
