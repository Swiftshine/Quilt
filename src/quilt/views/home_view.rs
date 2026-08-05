use crate::quilt::{docking::QuiltViewerTab, views::QuiltViewer};

impl QuiltViewer {
    pub fn show_home_ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top(ui.next_auto_id())
            .resizable(false)
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Editors", |ui| {
                        if ui.button("Level Editor").clicked() {
                            self.schedule_open_tab(QuiltViewerTab::LevelEditor);
                        }
                    });
                });
            });
    }
}
