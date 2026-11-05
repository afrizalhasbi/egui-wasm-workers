use crate::panels::N_PRIMES;
use engine::count_primes_in_range;
use std::time::Instant;

pub struct RayonDemo {
    pub serial_ms: Option<f64>,
    pub serial_count: Option<u64>,
    /// Ok = milliseconds, Err = reason parallel failed.
    pub parallel_result: Option<Result<f64, String>>,
    pub parallel_count: Option<u64>,
}

impl Default for RayonDemo {
    fn default() -> Self {
        Self {
            serial_ms: None,
            serial_count: None,
            parallel_result: None,
            parallel_count: None,
        }
    }
}

fn run_serial() -> (u64, f64) {
    let start = Instant::now();
    let count = count_primes_in_range(2, N_PRIMES);
    (count, start.elapsed().as_secs_f64() * 1000.0)
}

pub fn view(demo: &mut RayonDemo, ui: &mut egui::Ui) {
    ui.heading(format!(
        "4. Data-parallel job (prime count up to {N_PRIMES})"
    ));
    ui.label("Serial baseline always available; parallel via rayon on native.");

    ui.horizontal(|ui| {
        if ui.button("Run serial").clicked() {
            let (count, ms) = run_serial();
            demo.serial_count = Some(count);
            demo.serial_ms = Some(ms);
        }
        if let (Some(count), Some(ms)) = (demo.serial_count, demo.serial_ms) {
            ui.monospace(format!("serial: {count} primes in {ms:.1} ms"));
        }
    });

    // Parallel is native-only: wasm32 atomics need a nightly toolchain with
    // build-std, so the wasm build reports it as unavailable instead.
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    ui.horizontal(|ui| {
        if ui.button("Run parallel").clicked() {
            use rayon::prelude::*;
            let chunks = engine::prime_chunks(N_PRIMES);
            let start = Instant::now();
            let count: u64 = chunks
                .par_iter()
                .map(|&(lo, hi)| count_primes_in_range(lo, hi))
                .sum();
            demo.parallel_count = Some(count);
            demo.parallel_result = Some(Ok(start.elapsed().as_secs_f64() * 1000.0));
        }
        if let (Some(count), Some(Ok(ms))) = (demo.parallel_count, &demo.parallel_result) {
            ui.monospace(format!("parallel: {count} primes in {ms:.1} ms"));
        } else if let Some(Err(err)) = &demo.parallel_result {
            ui.colored_label(egui::Color32::LIGHT_RED, format!("parallel failed: {err}"));
        }
    });

    #[cfg(any(
        all(target_arch = "wasm32", feature = "parallel"),
        not(feature = "parallel")
    ))]
    ui.weak(
        "parallel: unavailable in this build (wasm32 atomics require a nightly \
         toolchain with build-std; native builds default to --features parallel).",
    );
}
