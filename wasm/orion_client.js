import * as wasm from "./orion_client_bg.wasm";
import { __wbg_set_wasm } from "./orion_client_bg.js";
__wbg_set_wasm(wasm);
export * from "./orion_client_bg.js";

wasm.__wbindgen_start();
