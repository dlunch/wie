import { Probe } from "./browser-host/pkg/wie_arm_browser_test.js";
import * as wasm from "./browser-host/pkg/wie_arm_browser_test_bg.wasm";

globalThis.backendTest = { Probe, wasm };
