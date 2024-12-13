use anyhow::Result;

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

                // image list list
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
                
                // all
                let _ = self.render_all_images(ui, rect);

                // layer-by-layer
                for layer in 0..12 {
                    self.render_by_layer(ui, rect, layer, egui::Vec2::splat(0.0));
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

    pub fn render_all_images(&mut self, ui: &mut egui::Ui, rect: egui::Rect) -> Result<()> {
        let bgst_file = self.bgst_renderer.bgst_file.as_ref().unwrap();
        // before rendering, collect entries based on
        // whether or not a mask is applied

        let (mut masked, mut unmasked): (Vec<BGSTEntry>, Vec<BGSTEntry>) = bgst_file
            .bgst_entries
            .iter()
            .partition(|entry| entry.main_image_index > -1 && entry.mask_image_index > -1);

        // sort both vectors by entry layer
        masked.sort_by(|a, b| a.layer.cmp(&b.layer));
        unmasked.sort_by(|a, b| a.layer.cmp(&b.layer));

        let painter = ui.painter_at(rect);
        let tile_size = 64.0;

        // render unmasked entries
        for entry in unmasked.iter() {
            // get the entry that isn't invalid
            let index = std::cmp::max(entry.main_image_index, entry.mask_image_index);
            
            if index < 0 || index as usize >= self.bgst_renderer.decoded_image_handles.len() {
                // both are invalid
                continue;
            }

            let tex_handle = &self.bgst_renderer.decoded_image_handles[index as usize];
            let tile_pos = egui::Vec2::new(
                entry.grid_x_position as f32,
                entry.grid_y_position as f32,
            );

            let render_pos = tile_pos + (tile_pos * tile_size);
            let tile_rect = egui::Rect::from_min_size(
                render_pos.to_pos2(),
                egui::Vec2::splat(tile_size)
            );


            
            painter.image(
                tex_handle.id(),
                tile_rect,
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                egui::Color32::WHITE
            );
            
            let resp = ui.interact(
                tile_rect,
                egui::Id::new(tex_handle as *const _),
                egui::Sense::click()
            );

            if resp.hovered() {
                painter.rect_filled(
                    tile_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x8)
                );
            }
        }

        // render masked entries
        for entry in masked.iter() {
            let main_index = entry.main_image_index as usize;
            let mask_index = entry.mask_image_index as usize;

            let masked_texture = self.bgst_renderer.masked_textures.get(&(main_index, mask_index));

            let tile_pos = egui::Vec2::new(
                entry.grid_x_position as f32,
                entry.grid_y_position as f32,
            );

            let render_pos = tile_pos + (tile_pos * tile_size);
            let tile_rect = egui::Rect::from_min_size(
                render_pos.to_pos2(),
                egui::Vec2::splat(tile_size)
            );

            if let Some(tex) = masked_texture {
                painter.image(
                    tex.id(),
                    tile_rect,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                    egui::Color32::WHITE
                );

                let resp = ui.interact(
                    tile_rect,
                    egui::Id::new(tex as *const _),
                    egui::Sense::click()
                );
                
                if resp.hovered() {
                    painter.rect_filled(
                        tile_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x8)
                    );
                }
            }
        }

        Ok(())
    }

    fn render_by_layer(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        layer: usize,
        origin: egui::Vec2
    ) {
        let bgst_file = self.bgst_renderer.bgst_file.as_ref().unwrap();

        let entries: Vec<&BGSTEntry> = Vec::from_iter(bgst_file.bgst_entries
            .iter()
            .filter(|entry| entry.layer as usize == layer));
        
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
