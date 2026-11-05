pub struct FileDropDemo {
    /// First 200 chars of the accepted file, if any.
    pub text: Option<String>,
    tx: std::sync::mpsc::Sender<Result<String, String>>,
    /// Results of async file reads (wasm reads files asynchronously).
    rx: std::sync::mpsc::Receiver<Result<String, String>>,
    received_files: usize,
}

impl Default for FileDropDemo {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            text: None,
            tx,
            rx,
            received_files: 0,
        }
    }
}

pub fn view(demo: &mut FileDropDemo, ui: &mut egui::Ui) {
    ui.heading("3. File drag-in");
    ui.label("Drag a plaintext file from your OS onto this window.");

    // Poll results of async reads (wasm).
    while let Ok(res) = demo.rx.try_recv() {
        demo.text = Some(res.unwrap_or_else(|err| format!("<read failed: {err}>")));
        demo.received_files += 1;
    }

    let (hovered, dropped) = ui
        .ctx()
        .input(|i| (i.raw.hovered_files.len(), i.raw.dropped_files.clone()));

    if !dropped.is_empty() {
        // Take only the first file per drop event. `dropped_files` is only
        // populated for the frame the backend reports the drop.
        let file = dropped[0].clone();
        let name = file.path().display().to_string();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let res = file
                .bytes()
                .map(|bytes| decode(&bytes, &name))
                .unwrap_or_else(|err| format!("<read failed: {err}>"));
            demo.text = Some(res);
            demo.received_files += 1;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let tx = demo.tx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let res = file.bytes_async().await.map(|bytes| decode(&bytes, &name));
                let _ = tx.send(res);
            });
        }
    }

    // Drop zone.
    let size = egui::Vec2::new(ui.available_width(), 80.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let visuals = ui.style().visuals.widgets.inactive.clone();
    let (fill, stroke) = if hovered > 0 {
        (
            egui::Color32::from_additive_luminance(40),
            visuals.fg_stroke,
        )
    } else {
        (egui::Color32::from_additive_luminance(8), visuals.bg_stroke)
    };
    ui.painter().rect(
        rect,
        egui::CornerRadius::same(6),
        fill,
        stroke,
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        if hovered > 0 {
            "release to load"
        } else {
            "drop a plaintext file here"
        },
        egui::TextStyle::Body.resolve(ui.style()),
        ui.visuals().text_color(),
    );

    // Full-window hover overlay.
    if hovered > 0 {
        let screen = ui.max_rect();
        let painter = ui.ctx().layer_painter(egui::LayerId::debug());
        painter.rect_filled(screen, 0.0, egui::Color32::from_black_alpha(100));
        painter.text(
            screen.center(),
            egui::Align2::CENTER_CENTER,
            "drop it",
            egui::TextStyle::Heading.resolve(ui.style()),
            egui::Color32::WHITE,
        );
    }

    ui.horizontal(|ui| {
        if let Some(text) = &demo.text {
            ui.monospace(format!("first 200 chars: {text}"));
            if ui.button("Delete text").clicked() {
                demo.text = None;
            }
        } else {
            ui.weak("(no file loaded)");
        }
    });
}

fn decode(bytes: &[u8], name: &str) -> String {
    let truncated: String = String::from_utf8_lossy(&bytes[..bytes.len().min(200)])
        .chars()
        .take(200)
        .collect();
    format!("[{name}] {truncated}")
}
