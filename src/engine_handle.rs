//! Non-blocking compute-engine handle.
//!
//! Native: a `std::thread` plus atomics.
//! Wasm: a Web Worker plus a `SharedArrayBuffer` (counter at bytes 0..8, stop
//! flag at byte 8), driven by `Atomics.wait`/`Atomics.notify`. The UI thread
//! never blocks: it reads the counter straight out of the SAB each frame.

pub trait Engine: Default {
    /// Toggle between running and stopped.
    fn toggle(&mut self);
    fn counter(&self) -> i64;
    fn running(&self) -> bool;
}

// ---------------------------------------------------------------- native ---

#[cfg(not(target_arch = "wasm32"))]
pub use native_impl::EngineImpl;

#[cfg(not(target_arch = "wasm32"))]
mod native_impl {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
    use std::thread::JoinHandle;

    pub struct EngineImpl {
        counter: std::sync::Arc<AtomicI64>,
        stop: std::sync::Arc<AtomicBool>,
        running: std::sync::Arc<AtomicBool>,
        handle: Option<JoinHandle<()>>,
    }

    impl Default for EngineImpl {
        fn default() -> Self {
            Self {
                counter: std::sync::Arc::new(AtomicI64::new(0)),
                stop: std::sync::Arc::new(AtomicBool::new(true)),
                running: std::sync::Arc::new(AtomicBool::new(false)),
                handle: None,
            }
        }
    }

    impl Engine for EngineImpl {
        fn toggle(&mut self) {
            if self.running() {
                self.stop.store(true, Ordering::Relaxed);
                self.running.store(false, Ordering::Relaxed);
                if let Some(handle) = self.handle.take() {
                    let _ = handle.join();
                }
            } else {
                self.stop.store(false, Ordering::Relaxed);
                self.running.store(true, Ordering::Relaxed);
                let counter = self.counter.clone();
                let stop = self.stop.clone();
                let running = self.running.clone();
                self.handle = Some(std::thread::spawn(move || {
                    let mut c = counter.load(Ordering::Relaxed);
                    while !stop.load(Ordering::Relaxed) {
                        c = engine::step(c);
                        counter.store(c, Ordering::Relaxed);
                    }
                    running.store(false, Ordering::Relaxed);
                }));
            }
        }

        fn counter(&self) -> i64 {
            self.counter.load(Ordering::Relaxed)
        }

        fn running(&self) -> bool {
            self.running.load(Ordering::Relaxed)
        }
    }

    impl EngineImpl {
        /// Workers don't exist natively; never an error.
        pub fn last_error(&self) -> Option<String> {
            None
        }
    }
}

// ------------------------------------------------------------------- wasm ---

#[cfg(target_arch = "wasm32")]
pub use wasm_impl::EngineImpl;

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    /// SAB layout (in i32 units for the 32-bit views):
    ///   [0] counter low, [1] counter high, [2] running flag (0 = run, 1 = stop)
    const SAB_BYTES: u32 = 16;
    const FLAG_IDX: u32 = 2;

    pub struct EngineImpl {
        sab: js_sys::SharedArrayBuffer,
        i32_view: js_sys::Int32Array,
        worker: Option<web_sys::Worker>,
        running: bool,
        /// True once the worker acked receipt of the SAB.
        worker_ack: std::rc::Rc<std::cell::RefCell<bool>>,
        /// Last worker error, shown in the UI (workers can't write to the
        /// page console from here, so we surface it in-app).
        last_error: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    }

    impl Default for EngineImpl {
        fn default() -> Self {
            let sab = js_sys::SharedArrayBuffer::new(SAB_BYTES);
            let i32_view = js_sys::Int32Array::new(&sab);
            // SAB is zero-initialized, and 0 means "run": park the worker in
            // the stopped state until the first toggle.
            let _ = js_sys::Atomics::store(&i32_view, FLAG_IDX, 1);
            let mut this = Self {
                sab,
                i32_view,
                worker: None,
                running: false,
                worker_ack: std::rc::Rc::new(std::cell::RefCell::new(false)),
                last_error: std::rc::Rc::new(std::cell::RefCell::new(None)),
            };
            // Start the worker in the stopped state; it blocks in Atomics.wait
            // until we set the flag and notify.
            this.ensure_worker();
            this
        }
    }

    impl EngineImpl {
        fn ensure_worker(&mut self) {
            if self.worker.is_some() {
                return;
            }
            let options = web_sys::WorkerOptions::new();
            options.set_type(web_sys::WorkerType::Module);
            let spawned = web_sys::Worker::new_with_options("./engine_worker.js", &options);
            let worker = match spawned {
                Ok(worker) => worker,
                Err(err) => {
                    let msg = format!("engine worker spawn failed: {err:?}");
                    web_sys::console::error_1(&msg.clone().into());
                    *self.last_error.borrow_mut() = Some(msg);
                    return;
                }
            };
            // Surface worker script errors in the UI, not just the console.
            let on_err = self.last_error.clone();
            let err_closure: Closure<dyn FnMut(JsValue)> =
                Closure::wrap(Box::new(move |e: JsValue| {
                    // ErrorEvent fields: message, filename, lineno.
                    let get = |k: &str| js_sys::Reflect::get(&e, &k.into()).ok();
                    let msg = get("message")
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let file = get("filename")
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let line = get("lineno")
                        .and_then(|v| v.as_f64())
                        .map(|l| l.to_string())
                        .unwrap_or_default();
                    *on_err.borrow_mut() =
                        Some(format!("worker error: {msg} ({file}:{line})"));
                    web_sys::console::error_1(
                        &format!("[engine] worker error event: {e:?}").into(),
                    );
                }));
            worker.set_onerror(Some(err_closure.as_ref().unchecked_ref()));
            err_closure.forget();
            let on_msg_err = self.last_error.clone();
            let msg_err_closure: Closure<dyn FnMut(JsValue)> =
                Closure::wrap(Box::new(move |e: JsValue| {
                    *on_msg_err.borrow_mut() = Some(format!("worker message error: {e:?}"));
                }));
            worker.set_onmessageerror(Some(msg_err_closure.as_ref().unchecked_ref()));
            msg_err_closure.forget();
            // Worker acks the SAB so the UI can show the handshake state.
            let on_ack = self.worker_ack.clone();
            let ack_closure: Closure<dyn FnMut(JsValue)> = Closure::wrap(Box::new(
                move |_e: JsValue| {
                    *on_ack.borrow_mut() = true;
                },
            ));
            worker.set_onmessage(Some(ack_closure.as_ref().unchecked_ref()));
            ack_closure.forget();
            // Hand the shared buffer to the worker once.
            let payload = self.sab.clone().unchecked_into::<JsValue>();
            if let Err(err) = worker.post_message(&payload) {
                web_sys::console::error_1(
                    &format!("engine worker postMessage failed: {err:?}").into(),
                );
            }
            self.worker = Some(worker);
        }

        /// Last worker error, if any.
        pub fn last_error(&self) -> Option<String> {
            self.last_error.borrow().clone()
        }

        /// "acked" once the worker confirmed SAB receipt.
        pub fn worker_state(&self) -> String {
            if *self.worker_ack.borrow() {
                "acked".into()
            } else if self.worker.is_some() {
                "spawned, no ack".into()
            } else {
                "spawn failed".into()
            }
        }

        /// One-line view of the SAB as the UI thread sees it.
        pub fn diagnostics(&self) -> String {
            let load = |idx: u32| {
                js_sys::Atomics::load(&self.i32_view, idx).unwrap_or(i32::MIN)
            };
            format!(
                "worker: {} | running: {} | flag: {} | sab lo/hi: {}/{}",
                self.worker_state(),
                self.running,
                load(FLAG_IDX),
                load(0),
                load(1)
            )
        }
    }

    impl Engine for EngineImpl {
        fn toggle(&mut self) {
            self.running = !self.running;
            self.ensure_worker();
            let flag = if self.running { 0 } else { 1 };
            let _ = js_sys::Atomics::store(&self.i32_view, FLAG_IDX, flag);
            let _ = js_sys::Atomics::notify(&self.i32_view, FLAG_IDX);
        }

        fn counter(&self) -> i64 {
            // Read via the i32 view (load_bigint/BigInt64Array is flaky here);
            // compose lo/hi into an i64, same layout the worker writes.
            let lo = js_sys::Atomics::load(&self.i32_view, 0).unwrap_or(0) as u32 as i64;
            let hi = js_sys::Atomics::load(&self.i32_view, 1).unwrap_or(0) as u32 as i64;
            (hi << 32) | (lo & 0xFFFF_FFFF)
        }

        fn running(&self) -> bool {
            self.running
        }
    }
}
