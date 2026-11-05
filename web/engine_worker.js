// Engine worker: loads the SAME wasm module a second time and runs the
// engine loop inside Atomics.wait on a SharedArrayBuffer.
//
// SAB layout (i32 units): [0] counter low, [1] counter high, [2] running flag
// (0 = run, 1 = stop).
import __wbg_init, { engine_step } from "./pkg/egui_wasm_workers.js";

console.log("[engine_worker] module prologue running");

const FLAG = 2;

function loop(i32) {
  const readCounter = () => {
    const lo = Atomics.load(i32, 0) >>> 0;
    const hi = Atomics.load(i32, 1) >>> 0;
    return BigInt.asIntN(64, (BigInt(hi) << 32n) | BigInt(lo));
  };
  const writeCounter = (c) => {
    Atomics.store(i32, 0, Number(BigInt.asIntN(32, c)));
    Atomics.store(i32, 1, Number(BigInt.asIntN(32, c >> 32n)));
  };

  while (true) {
    if (Atomics.load(i32, FLAG) !== 0) {
      // Park until the UI thread notifies.
      Atomics.wait(i32, FLAG, 1);
      continue;
    }
    // Tight increment loop; the stop flag is an atomic load, so the store +
    // notify from the UI thread takes effect on the next iteration.
    writeCounter(BigInt.asIntN(64, engine_step(BigInt.asUintN(64, readCounter()))));
  }
}

// Assigned in the module's synchronous prologue, BEFORE the top-level await,
// so the SAB message can never be dispatched to a handler-less worker. The
// loop only starts once BOTH the wasm init and the SAB have arrived.
let i32 = null;
let initialized = false;
let started = false;

function maybeStart() {
  if (initialized && i32 && !started) {
    started = true;
    loop(i32);
  }
}

onmessage = (e) => {
  console.log("[engine_worker] onmessage fired");
  if (!i32) {
    i32 = new Int32Array(e.data);
    console.log("[engine_worker] SAB received, acking");
    postMessage("sab-ok"); // ack for the UI's handshake readout
  }
  maybeStart();
};

console.log("[engine_worker] awaiting wasm init");
await __wbg_init();
console.log("[engine_worker] wasm init done");
initialized = true;
maybeStart();
console.log("[engine_worker] module evaluation complete");
