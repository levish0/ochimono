/* @ts-self-types="./ochimono_engine.d.ts" */
import * as wasm from "./ochimono_engine_bg.wasm";
import { __wbg_set_wasm } from "./ochimono_engine_bg.js";

__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    Game
} from "./ochimono_engine_bg.js";
