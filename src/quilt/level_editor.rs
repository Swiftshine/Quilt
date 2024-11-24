use egui;

#[derive(Default)]
pub struct LevelEditor;

impl LevelEditor {
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui|{
            ui.label("Level editor.");
        });
    }
}
