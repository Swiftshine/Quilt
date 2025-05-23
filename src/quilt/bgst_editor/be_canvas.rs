
use crate::quilt::game::bgst::BGSTEntry;

use super::BGSTEditor;

impl BGSTEditor {
    pub fn render_contents(&mut self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style())
        .show(ui, |ui|{
            egui::ScrollArea::both().id_salt("be_image_render").show(ui, |ui|{
                let desired_size = ui.available_size();
                let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

                // image list
                egui::Area::new(egui::Id::from("be_image_list"))
                .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
                .show(ui.ctx(), |ui|{
                    egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui|{
                        ui.collapsing("Image list", |ui|{
                            self.display_image_list(ui);
                        });
                    });
                });

                // grid rendering
                let origin = egui::Vec2::splat(8.0);

                // layer-by-layer
                for layer in 0..12 {
                    self.render_by_layer(ui, rect, layer, origin);
                }
            });
        });
    }

    pub fn display_image_list(&mut self, ui: &mut egui::Ui) {
        let bgst_file = self.bgst_renderer.bgst_file.as_ref().unwrap();
        ui.label(format!("Count: {}", bgst_file.compressed_images.len()));
        
        let table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto()) // image column
            .column(egui_extras::Column::auto()); // index column

        table.body(|mut body| {
            for (index, image_handle) in self.bgst_renderer.decoded_image_handles.iter().enumerate() {
                body.row(32.0, |mut row| {
                    // image column
                    row.col(|ui| {
                        ui.add(
                            egui::Image::new(image_handle).max_size(egui::Vec2::splat(100.0))
                        );
                    });
                    // index column
                    row.col(|ui| {
                        ui.label(format!("Index {index}"));
                    });
                });
            }
        });
    }

    fn render_by_layer(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        layer: i16,
        origin: egui::Vec2
    ) {
        let bgst_file = self.bgst_renderer.bgst_file.as_ref().unwrap();

        let entries: Vec<&BGSTEntry> = Vec::from_iter(bgst_file.bgst_entries
            .iter()
            .filter(|entry| entry.layer == layer));
        
        // let entries = Vec::from_iter(bgst_file.bgst_entries.iter());

        // separate by mask

        let (masked, unmasked): (Vec<&BGSTEntry>, Vec<&BGSTEntry>) =
            entries
            .iter()
            .partition(|entry| entry.main_image_index > -1 && entry.mask_image_index > -1);

        
        // unmasked
        for entry in unmasked {
            self.bgst_renderer.render_unmasked_entry(ui, rect, entry, origin);
        }
        
        // masked
        for entry in masked {
            self.bgst_renderer.render_masked_entry(ui, rect, entry, origin);
        }
    }
}
