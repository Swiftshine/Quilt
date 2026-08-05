use crate::quilt::views::QuiltViewer;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum QuiltViewerTab {
    Home,
    LevelEditor,
    // BGSTEditor,
}

impl QuiltViewerTab {
    fn get_name(&self) -> String {
        // todo: emojis eventually
        match self {
            Self::Home => "Home",
            Self::LevelEditor => "Level Editor",
            // Self::BGSTEditor => "BGST Editor",
        }
        .to_string()
    }
}

pub struct QuiltViewerTabViewer<'a> {
    quilt_viewer: &'a mut QuiltViewer,
}

impl<'a> QuiltViewerTabViewer<'a> {
    pub fn new(quilt_viewer: &'a mut QuiltViewer) -> Self {
        Self { quilt_viewer }
    }
}

impl<'a> egui_dock::TabViewer for QuiltViewerTabViewer<'a> {
    type Tab = QuiltViewerTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        tab.get_name().into()
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        *tab != Self::Tab::Home
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        *tab != Self::Tab::Home
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Self::Tab::Home => {
                self.quilt_viewer.show_home_ui(ui);
            }

            Self::Tab::LevelEditor => {
                self.quilt_viewer.level_editor.show_ui(ui);
            }
        }
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::Id {
        egui_dock::egui::Id::new(tab)
    }
}

impl QuiltViewer {
    pub fn update_dock(&mut self, ui: &mut egui::Ui) {
        // temporarily move dock state out to avoid borrowing self twice
        let mut dock_state = self.dock_state.take().unwrap();

        egui_dock::DockArea::new(&mut dock_state)
            .style(egui_dock::Style::from_egui(ui.style()))
            .id(ui.auto_id_with("qv_dock"))
            .show_inside(ui, &mut QuiltViewerTabViewer::new(self));

        // put it back
        self.dock_state = Some(dock_state);

        // tab adding
        if let Some(tab) = self.tab_to_open.take() {
            self.open_tab(tab);
        }
    }

    // needs to be delayed due to ownership of the dock state
    pub fn schedule_open_tab(&mut self, tab: QuiltViewerTab) {
        self.tab_to_open = Some(tab);
    }

    pub fn open_tab(&mut self, tab: QuiltViewerTab) {
        let found = {
            self.dock_state
                .as_ref()
                .unwrap()
                .main_surface()
                .iter()
                .any(|node| node.tabs().is_some_and(|tabs| tabs.contains(&tab)))
        };

        if !found {
            self.dock_state
                .as_mut()
                .unwrap()
                .main_surface_mut()
                .push_to_first_leaf(tab);
        }
    }
}
