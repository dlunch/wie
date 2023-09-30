import { start } from "./pkg";

const main = () => {
  const file = document.getElementById("file") as HTMLInputElement;
  const button = document.getElementById("start") as HTMLButtonElement;

  button.addEventListener("click", () => {
    let selected_file = file.files[0];

    if (selected_file) {
      let reader = new FileReader();

      reader.onload = (e) => {
        let data = e.target?.result as ArrayBuffer;

        start(new Uint8Array(data));
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
