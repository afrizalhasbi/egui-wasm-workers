fn main() {
    if let Err(err) = egui_wasm_workers::run_native() {
        eprintln!("eframe error: {err}");
    }
}
