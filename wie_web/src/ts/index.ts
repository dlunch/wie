import { WieWeb } from "./pkg";

const main = () => {
  const file = document.getElementById("file") as HTMLInputElement;
  const button = document.getElementById("start") as HTMLButtonElement;

  button.addEventListener("click", () => {
    let selected_file = file.files[0];

    if (selected_file) {
      let reader = new FileReader();

      reader.onload = (e) => {
        let data = e.target?.result as ArrayBuffer;

        try {
          const canvas = document.getElementById("canvas") as HTMLCanvasElement;
          const wie_web = new WieWeb(new Uint8Array(data), canvas);

          for (const button of document.querySelectorAll("button[data-key]")) {
            button.addEventListener("mousedown", (e) => {
              const key = (e.target as HTMLButtonElement).dataset.key;
              wie_web.send_key(key);
            });
          }
          let update = () => {
            try {
              wie_web.update();
            } catch (e) {
              wie_web.free();
              alert(e.message);
              throw e;
            }

            requestAnimationFrame(update);
          };

          requestAnimationFrame(update);
        } catch (e) {
          alert(e.message);
          throw e;
        }
      };

      reader.readAsArrayBuffer(selected_file);
    }
  });
};

if (document.readyState !== "loading") {
  main();
} else {
  document.addEventListener("DOMContentLoaded", () => main());
}
