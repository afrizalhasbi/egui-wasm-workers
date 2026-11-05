mod counter;
mod dnd_boxes;
mod file_drop;
mod rayon;

pub use dnd_boxes::DndDemo;
pub use file_drop::FileDropDemo;
pub use rayon::RayonDemo;

pub const N_PRIMES: u64 = 2_000_000;

pub fn view(model: &mut crate::Model, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        counter::view(&mut model.engine, ui);
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        dnd_boxes::view(&mut model.dnd, ui);
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        file_drop::view(&mut model.file_drop, ui);
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        rayon::view(&mut model.rayon, ui);
    });
}
