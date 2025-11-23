use crate::quilt::game::bgst::BGSTEntry;

use super::BGSTEditor;

impl BGSTEditor {
    pub fn render_contents(&mut self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style())
        .show(ui, |ui|{
            egui::ScrollArea::both().id_salt("be_image_render_scroll_area").show(ui, |ui|{
                let desired_size = ui.available_size();
                let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
                
                // let origin = response.hover_pos().unwrap_or_default();

                // TODO/NOTE:

                // the origin is in the top left.

                // let origin = egui::Vec2::splat(500.0);


                // get grid origin
                
                let origin = self.find_grid_origin();

                // let origin = egui::Vec2::new(30.0, 80.0);
                

                self.bgst_renderer.zoom = 5.0f32;

                // grid rendering

                // layer-by-layer

                for layer in 0..12 {
                    if self.layer_rendered[layer] {
                        // self.render_by_layer(ui, rect, layer as i16, origin.to_vec2());
                        self.render_by_layer(ui, rect, layer as i16, origin);
                    }
                }


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

                // layer toggles
                egui::Area::new(egui::Id::from("be_layer_toggle"))
                .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(10.0, 10.0))
                .show(ui.ctx(), |ui|{
                    egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui|{
                        ui.collapsing("Toggle Layers", |ui|{
                            for layer in 0..12 {
                                let text = format!("Layer {}", layer + 1);
                                ui.checkbox(&mut self.layer_rendered[layer], text);
                            }
                        });
                    });
                
                });

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

        println!("BGSTEditor::render_by_layer - origin: {}", origin);

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

    fn find_grid_origin(&self) -> egui::Vec2 {
        // origin is the top left corner of the grid

        let bgst_file = &self.bgst_renderer.bgst_file.as_ref().unwrap();

        let _grid_height = bgst_file.grid_height;
        let _grid_width = bgst_file.grid_width;

        let _image_height = bgst_file.image_height;
        let _image_width = bgst_file.image_width;

        
        egui::Vec2::new(0.0f32, 0.0f32)
    }
    
}
