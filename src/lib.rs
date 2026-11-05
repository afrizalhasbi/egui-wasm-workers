pub mod engine_handle;
pub mod panels;

#[allow(dead_code)]
enum Message {}

/// Shared application state. Elm-ish: `panels::view` mutates state and the
/// egui ui in one pass.
pub struct Model {
    pub engine: engine_handle::EngineImpl,
    pub dnd: panels::DndDemo,
    pub file_drop: panels::FileDropDemo,
    pub rayon: panels::RayonDemo,
    pub stats: Stats,
}

#[derive(Default)]
pub struct Stats {
    pub received_files: usize,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            engine: engine_handle::EngineImpl::default(),
            dnd: panels::DndDemo::default(),
            file_drop: panels::FileDropDemo::default(),
            rayon: panels::RayonDemo::default(),
            stats: Stats::default(),
        }
    }
}

impl eframe::App for Model {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ui, |ui| {
            panels::view(self, ui);
        });
    }
}

/// Native entry point.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui-workers",
        options,
        Box::new(|_cc| Ok(Box::new(Model::default()))),
    )
}

// --------------------------------------------------------------- wasm ---

#[cfg(target_arch = "wasm32")]
pub use wasm_exports::*;

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn engine_step(counter: i64) -> i64 {
        // Same engine logic the native thread runs, exported for the worker.
        engine::step(counter)
    }

    #[wasm_bindgen]
    pub struct WebHandle {
        runner: eframe::WebRunner,
    }

    #[wasm_bindgen]
    impl WebHandle {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                runner: eframe::WebRunner::new(),
            }
        }

        /// Call once from JavaScript with the app canvas.
        pub async fn start(
            &self,
            canvas: web_sys::HtmlCanvasElement,
        ) -> Result<(), wasm_bindgen::JsValue> {
            self.runner
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|_cc| Ok(Box::new(crate::Model::default()))),
                )
                .await
        }
    }
}
