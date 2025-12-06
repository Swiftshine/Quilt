// use super::bgst_renderer::BGSTRenderer;
// mod be_io;
// mod be_canvas;

// #[derive(Default)]
// pub struct BGSTEditor {
//     bgst_renderer: BGSTRenderer,
//     layer_rendered: [bool; 12],
// }

// impl BGSTEditor {
//     pub fn new() -> Self {
//         Self {
//             bgst_renderer: BGSTRenderer::new(),
//             layer_rendered: [true; 12],
//         }
//     }

//     pub fn show_ui(&mut self, ui: &mut egui::Ui) {
//         egui::TopBottomPanel::top("be_top_panel")
//         .show(ui.ctx(), |ui|{
//             egui::menu::bar(ui, |ui|{
//                 if ui.button("Open").clicked() {
//                     let _ = self.bgst_renderer.open_file(ui);
//                     ui.close_menu();
//                 }

//                 // if ui.add_enabled(self.file_open, egui::Button::new("Save"))
//                 // .clicked() {
//                 //     // save file
//                 //     ui.close_menu();
//                 // }

//                 // if ui.add_enabled(self.file_open, egui::Button::new("Save as"))
//                 // .clicked() {
//                 //     // save file as
//                 //     ui.close_menu();
//                 // }
//             });
//         });

//         egui::CentralPanel::default()
//         .show(ui.ctx(), |ui|{
//             if self.bgst_renderer.bgst_file.is_some() {
//                 self.render_contents(ui);
//             }
//         });
//     }
// }
