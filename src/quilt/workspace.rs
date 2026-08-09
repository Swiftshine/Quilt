use crate::quilt::{
    archive_viewer::ArchiveViewer, bgst_editor::BGSTEditor, docking::WorkspaceManagerTab,
};

use slotmap::{SlotMap, new_key_type};

new_key_type! {
    pub struct WorkspaceID;
}

pub struct Workspace {
    pub name: String,
    pub archive_viewer: ArchiveViewer,
    pub bgst_editor: BGSTEditor,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            name: "New Workspace".to_string(),
            archive_viewer: ArchiveViewer::new(),
            bgst_editor: BGSTEditor::new(),
        }
    }

    pub fn on_exit(&mut self) {
        println!("Workspace::on_exit()");
    }

    pub fn current_editor_type(&self) -> &'static str {
        "None"
    }
}

pub struct WorkspaceManager {
    pub workspaces: SlotMap<WorkspaceID, Workspace>,
    pub dock_state: Option<egui_dock::DockState<WorkspaceManagerTab>>,

    // things we're waiting to do
    pub pending_workspace_addition: bool,
    pub pending_workspace_deletion: Option<WorkspaceID>,
    pub pending_workspace_focus: Option<WorkspaceID>,
}

impl WorkspaceManager {
    fn default_dock() -> egui_dock::DockState<WorkspaceManagerTab> {
        egui_dock::DockState::new(vec![WorkspaceManagerTab::Home])
    }

    pub fn new() -> Self {
        let dock_state = Some(Self::default_dock());

        Self {
            dock_state,
            workspaces: SlotMap::default(),
            pending_workspace_addition: false,
            pending_workspace_deletion: None,
            pending_workspace_focus: None,
        }
    }

    pub fn on_exit(&mut self) {
        for (_, workspace) in self.workspaces.iter_mut() {
            workspace.on_exit();
        }
    }

    pub fn delete_workspace(&mut self, id: WorkspaceID) {
        // remove workspace from dock first

        if let Some(dock_state) = self.dock_state.as_mut() {
            if let Some(path) = dock_state.find_tab_from(
                |tab| matches!(tab, WorkspaceManagerTab::Workspace(tab_id) if *tab_id == id),
            ) {
                dock_state.remove_tab(path);
            }
        }

        self.workspaces.remove(id);
    }

    fn open_workspace_tab(&mut self, id: WorkspaceID) {
        if let Some(dock_state) = self.dock_state.as_mut() {
            dock_state
                .main_surface_mut()
                .push_to_first_leaf(WorkspaceManagerTab::Workspace(id));
        }
    }

    pub fn focus_workspace(&mut self, id: WorkspaceID) {
        if !self.workspaces.contains_key(id) {
            return;
        }

        if let Some(dock_state) = self.dock_state.as_mut() {
            if let Some(path) = dock_state.find_tab_from(
                |tab| matches!(tab, WorkspaceManagerTab::Workspace(tab_id) if *tab_id == id),
            ) {
                dock_state.set_focused_node_and_surface(path.node_path());
                let _ = dock_state.set_active_tab(path);
                return;
            }
        }

        // workspace exists but the tab was closed
        self.open_workspace_tab(id);

        if let Some(dock_state) = self.dock_state.as_mut() {
            if let Some(path) = dock_state.find_tab_from(
                |tab| matches!(tab, WorkspaceManagerTab::Workspace(tab_id) if *tab_id == id),
            ) {
                dock_state.set_focused_node_and_surface(path.node_path());
                let _ = dock_state.set_active_tab(path);
            }
        }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        self.update_dock(ui);
    }

    pub fn create_new_workspace(&mut self) -> WorkspaceID {
        let id = self.workspaces.insert(Workspace::new());

        self.dock_state
            .as_mut()
            .unwrap()
            .main_surface_mut()
            .push_to_first_leaf(WorkspaceManagerTab::Workspace(id));

        id
    }

    pub fn show_tab_ui(&mut self, ui: &mut egui::Ui) {
        self.show_workspace_list(ui);
    }

    fn show_workspace_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Workspaces");

            if ui.small_button("+").clicked() {
                self.pending_workspace_addition = true;
            }
        });

        ui.add_space(6.0);

        for (id, workspace) in self.workspaces.iter_mut() {
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.small_button("×").clicked() {
                            self.pending_workspace_deletion = Some(id);
                        }

                        if ui.small_button(">").clicked() {
                            self.pending_workspace_focus = Some(id);
                        }

                        egui::TextEdit::singleline(&mut workspace.name)
                            .hint_text("Enter a name for your workspace")
                            .desired_width(ui.available_width() - 20.0)
                            .show(ui);
                    });
                });
        }
    }
}
