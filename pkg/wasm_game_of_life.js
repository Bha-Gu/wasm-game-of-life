/* @ts-self-types="./wasm_game_of_life.d.ts" */
import * as wasm from "./wasm_game_of_life_bg.wasm";
import { __wbg_set_wasm } from "./wasm_game_of_life_bg.js";

__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    Cell, Universe
} from "./wasm_game_of_life_bg.js";
