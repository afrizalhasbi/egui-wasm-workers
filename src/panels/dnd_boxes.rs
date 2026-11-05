use egui::{Color32, CornerRadius, Rect, Sense, Stroke, Vec2};
use std::collections::HashSet;

pub struct DndDemo {
    /// tokens[i] = home box index of token i
    pub tokens: Vec<usize>,
    pub selected: HashSet<usize>,
}

impl Default for DndDemo {
    fn default() -> Self {
        Self {
            tokens: vec![0, 0],
            selected: HashSet::new(),
        }
    }
}

const BOX_SIZE: Vec2 = Vec2::new(200.0, 120.0);
const GAP: f32 = 24.0;
const TOKEN_SIZE: Vec2 = Vec2::new(70.0, 34.0);

pub fn view(demo: &mut DndDemo, ui: &mut egui::Ui) {
    ui.heading("2. Drag & drop boxes");
    ui.label("Drag a rectangle into the other box. Shift-click to select more than one.");

    let base = ui.available_rect_before_wrap().min;
    let r0 = Rect::from_min_size(base, BOX_SIZE);
    let r1 = Rect::from_min_size(base + Vec2::new(BOX_SIZE.x + GAP, 0.0), BOX_SIZE);
    let box_rects = [r0, r1];

    // Reserve layout space for both boxes.
    ui.allocate_space(Vec2::new(BOX_SIZE.x * 2.0 + GAP, BOX_SIZE.y));

    // Boxes: hover highlight when a drag payload is in flight.
    let dragging = egui::DragAndDrop::payload::<Vec<usize>>(ui.ctx()).is_some();
    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let mut drop_target: Option<usize> = None;
    for (bi, &rect) in box_rects.iter().enumerate() {
        let ui_st = ui.style();
        let mut fill = Color32::from_additive_luminance(12);
        let mut stroke = ui_st.visuals.widgets.inactive.bg_stroke;
        if dragging {
            if pointer_pos.is_some_and(|p| rect.contains(p)) {
                fill = Color32::from_additive_luminance(35);
                stroke = ui_st.visuals.widgets.active.bg_stroke;
                drop_target = Some(bi);
            }
        }
        ui.painter().rect(
            rect,
            CornerRadius::same(6),
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.left_top() + Vec2::new(10.0, 8.0),
            egui::Align2::LEFT_TOP,
            format!("box {bi}"),
            egui::TextStyle::Small.resolve(ui.style()),
            ui.visuals().weak_text_color(),
        );
    }

    // Tokens.
    let mut in_box = [0usize; 2];
    let selected_ids: Vec<usize> = egui::DragAndDrop::payload::<Vec<usize>>(ui.ctx())
        .map(|p| p.as_ref().clone())
        .unwrap_or_default();

    for (ti, &home) in demo.tokens.iter().enumerate() {
        let slot = in_box[home];
        in_box[home] += 1;
        let token_rect = Rect::from_min_size(
            box_rects[home].min + Vec2::new(12.0 + slot as f32 * (TOKEN_SIZE.x + 8.0), 30.0),
            TOKEN_SIZE,
        );

        let selected = demo.selected.contains(&ti);
        let (fill, stroke): (Color32, Stroke) = if selected {
            (
                Color32::from_rgb(80, 120, 200),
                Stroke::new(1.0, Color32::from_rgb(160, 200, 255)),
            )
        } else {
            (
                Color32::from_rgb(60, 60, 70),
                Stroke::new(1.0, Color32::from_rgb(120, 120, 130)),
            )
        };

        let resp = ui.allocate_rect(token_rect, Sense::click_and_drag());
        let hovered = resp.hovered();
        let fill = if hovered && !selected {
            Color32::from_rgb(90, 130, 210)
        } else {
            fill
        };
        // Draw ghosted token in its home slot while it is being dragged away.
        let being_dragged = dragging && selected_ids.contains(&ti);
        let alpha = if being_dragged { 0.3 } else { 1.0 };
        let fill = fill.gamma_multiply(alpha);
        let stroke = Stroke::new(1.0, stroke.color.gamma_multiply(alpha));
        ui.painter().rect(
            token_rect,
            CornerRadius::same(4),
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            token_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("rect {ti}"),
            egui::TextStyle::Body.resolve(ui.style()),
            ui.visuals().text_color().gamma_multiply(alpha),
        );

        // Click to select (shift toggles multi-select).
        if resp.clicked() {
            let shift = ui.input(|i| i.modifiers.shift);
            if shift {
                if demo.selected.contains(&ti) {
                    demo.selected.remove(&ti);
                } else {
                    demo.selected.insert(ti);
                }
            } else {
                demo.selected.clear();
                demo.selected.insert(ti);
            }
        }

        // Start dragging all selected tokens (make sure the grabbed one is in).
        if resp.drag_started() {
            if !demo.selected.contains(&ti) {
                demo.selected.clear();
                demo.selected.insert(ti);
            }
            let ids: Vec<usize> = demo.selected.iter().copied().collect();
            egui::DragAndDrop::set_payload(ui.ctx(), ids);
        }
    }

    // Ghost of the dragged tokens following the cursor.
    if dragging {
        if let Some(p) = pointer_pos {
            for (k, _) in selected_ids.iter().enumerate() {
                let ghost = Rect::from_min_size(
                    p + Vec2::new(
                        -TOKEN_SIZE.x / 2.0,
                        -TOKEN_SIZE.y / 2.0 + k as f32 * (TOKEN_SIZE.y + 4.0),
                    ),
                    TOKEN_SIZE,
                );
                ui.painter().rect(
                    ghost,
                    CornerRadius::same(4),
                    Color32::from_rgba_unmultiplied(80, 120, 200, 160),
                    Stroke::new(1.0, Color32::from_rgb(160, 200, 255)),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }

    // Resolve the drop on the release frame (payload is still readable then).
    if dragging
        && ui.input(|i| i.pointer.any_released())
        && let (Some(target), Some(ids)) = (
            drop_target,
            egui::DragAndDrop::payload::<Vec<usize>>(ui.ctx()),
        )
    {
        for &ti in ids.as_ref() {
            demo.tokens[ti] = target;
        }
        egui::DragAndDrop::clear_payload(ui.ctx());
    }
}
