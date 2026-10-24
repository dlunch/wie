import { WieWeb } from "@pkg";

import type { ScaleFilter, SessionEvent } from "../src/ts/backend";
import { startWasmSession, type WasmSession } from "../src/ts/backend/wasm";

let session: WasmSession | undefined;
let controller: AbortController;
let suspended = false;
let ticks = 0;
let disposals = 0;
const events: SessionEvent[] = [];
const update = WieWeb.prototype.update;
WieWeb.prototype.update = function () { ticks++; return update.call(this); };

const test = {
  async start(filter: ScaleFilter = "nearest", paused = false) {
    const [archive, font] = await Promise.all([
      fetch("/fixture.jar").then(response => response.arrayBuffer()),
      fetch("/font.ttf").then(response => response.arrayBuffer()),
    ]);
    controller = new AbortController();
    suspended = paused;
    events.length = 0;
    session = await startWasmSession("fixture.jar", new Uint8Array(archive), new Uint8Array(font),
      event => events.push(event), controller.signal, filter, {
        now: () => Date.now(),
        storage: async () => null,
        vibrate() {},
        audio: { play() {}, stop() {}, dispose() { disposals++; } },
      }, () => suspended);
    return !!session;
  },
  suspend(value: boolean) { suspended = value; session?.suspend(value); },
  async key() { await session!.key("1", true); await session!.key("1", false); },
  abort() { controller.abort(); },
  async stop() { await session?.stop(); session = undefined; },
  state() { return { ticks, disposals, events }; },
};

Object.assign(window, { presentationTest: test });
