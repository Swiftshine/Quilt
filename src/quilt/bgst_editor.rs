use super::bgst_renderer::BGSTRenderer;
mod be_io;

#[derive(Default)]
pub struct BGSTEditor {
    _bgst_renderer: BGSTRenderer,
    file_open: bool,
}

impl BGSTEditor {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("be_top_panel")
        .show(ui.ctx(), |ui|{
            egui::menu::bar(ui, |ui|{
                if ui.button("Open").clicked() {
                    // open file
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, egui::Button::new("Save"))
                .clicked() {
                    // save file
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, egui::Button::new("Save as"))
                .clicked() {
                    // save file as
                    ui.close_menu();
                }
            });
        });
    }
}