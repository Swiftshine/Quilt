use crate::quilt::workspace::{Workspace, WorkspaceID, WorkspaceManager};

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum WorkspaceTab {
    Home,
    LevelEditor,
    BGSTEditor,
}

impl WorkspaceTab {
    fn get_name(&self) -> String {
        // todo: emojis eventually
        match self {
            Self::Home => "Home",
            Self::LevelEditor => "Level Editor",
            Self::BGSTEditor => "BGST Editor",
        }
        .to_string()
    }
}

pub struct WorkspaceTabViewer<'a> {
    workspace: &'a mut Workspace,
}

impl<'a> WorkspaceTabViewer<'a> {
    pub fn new(workspace: &'a mut Workspace) -> Self {
        Self { workspace }
    }
}

impl Workspace {
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("workspace!");
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum WorkspaceManagerTab {
    Home,
    Workspace(WorkspaceID),
}

impl WorkspaceManagerTab {
    fn get_name(&self) -> String {
        match self {
            Self::Home => "Workspace Management".to_string(),
            Self::Workspace(id) => format!("Workspace: {:?}", id),
        }
    }
}

pub struct WorkspaceManagerTabViewer<'a> {
    workspace_manager: &'a mut WorkspaceManager,
}

impl<'a> WorkspaceManagerTabViewer<'a> {
    fn new(workspace_manager: &'a mut WorkspaceManager) -> Self {
        Self { workspace_manager }
    }
}

impl<'a> egui_dock::TabViewer for WorkspaceManagerTabViewer<'a> {
    type Tab = WorkspaceManagerTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Self::Tab::Home => "Home".to_string(),
            Self::Tab::Workspace(id) => {
                let workspace = &self.workspace_manager.workspaces[*id];
                workspace.name.to_owned()
            }
        }
        .into()
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab, _path: egui_dock::NodePath) {
        if let WorkspaceManagerTab::Workspace(id) = tab {
            if ui.button("Delete").clicked() {
                self.workspace_manager.pending_workspace_deletion = Some(*id);
            }
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Self::Tab::Home)
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Self::Tab::Home => {
                self.workspace_manager.show_tab_ui(ui);
            }

            Self::Tab::Workspace(workspace_id) => {
                if let Some(workspace) = self.workspace_manager.workspaces.get_mut(*workspace_id) {
                    workspace.show_ui(ui);
                } else {
                    ui.colored_label(egui::Color32::RED, "This workspace no longer exists.");
                }
            }
        }
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab)
    }
}

impl WorkspaceManager {
    pub fn update_dock(&mut self, ui: &mut egui::Ui) {
        let mut dock_state = self.dock_state.take().unwrap();

        egui_dock::DockArea::new(&mut dock_state)
            .style(egui_dock::Style::from_egui(ui.style()))
            .id(ui.auto_id_with("q_workspace_manager_dock"))
            .show_inside(ui, &mut WorkspaceManagerTabViewer::new(self));

        self.dock_state = Some(dock_state);

        // misc updates

        if self.pending_workspace_addition {
            self.create_new_workspace();
            self.pending_workspace_addition = false;
        }

        if let Some(id) = self.pending_workspace_deletion.take() {
            self.delete_workspace(id);
        }

        if let Some(id) = self.pending_workspace_focus.take() {
            self.focus_workspace(id);
        }
    }
}
