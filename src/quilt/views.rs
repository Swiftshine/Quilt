use egui::Ui;

use crate::quilt::{bgst_editor::BGSTEditor, docking::QuiltViewerTab, level_editor::LevelEditor};

#[derive(PartialEq)]
pub enum QuiltView {
    Home,
    LevelEditor,
    BGSTEditor,
    GfArchUtility,
}

pub struct QuiltViewer {
    // editors
    pub level_editor: LevelEditor,
    pub bgst_editor: BGSTEditor,

    // dock state
    pub dock_state: Option<egui_dock::DockState<QuiltViewerTab>>,
    pub tab_to_open: Option<QuiltViewerTab>,
}

impl QuiltViewer {
    pub fn default_dock() -> egui_dock::DockState<QuiltViewerTab> {
        egui_dock::DockState::new(vec![QuiltViewerTab::Home])
    }

    pub fn new() -> Self {
        let dock_state = Some(Self::default_dock());

        let mut viewer = Self {
            level_editor: LevelEditor::new(),
            bgst_editor: BGSTEditor::new(),
            dock_state,
            tab_to_open: None,
        };

        viewer.on_start();

        viewer
    }

    fn on_start(&mut self) {
        // nothing yet
        println!("QuiltViewer::on_start()")
    }

    pub fn on_exit(&mut self) {
        // nothing yet
        println!("QuiltViewer::on_exit()")
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        self.update_dock(ui);
    }

    pub fn show_home_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Home");
    }
}
