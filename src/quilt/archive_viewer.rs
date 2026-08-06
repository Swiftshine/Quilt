pub struct ArchiveViewer;

impl ArchiveViewer {
    pub fn new() -> Self {
        Self { }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Archive viewer");
    }
}
