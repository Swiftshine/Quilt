use std::collections::HashMap;

use crate::quilt::bgst_editor::TileSelection;

use super::BGSTEditor;

impl BGSTEditor {
    pub fn render_contents(&mut self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("be_image_render_scroll_area")
                .show(ui, |ui| {
                    // grid

                    egui::Grid::new("bg_image_grid")
                        .spacing(egui::Vec2::ZERO) // no padding in between squares
                        .show(ui, |ui| {
                            if let Some(bgst_file) = self.bgst_renderer.bgst_file.as_ref() {
                                let image_render_size = egui::Vec2::splat(100.0);

                                // we need to be able to draw empty squares should a coordinate not have
                                // any associated cells

                                // make a map for lookup

                                // (y, x), index
                                let mut entry_map: HashMap<(u32, u32), usize> = HashMap::new();

                                for (index, entry) in bgst_file.bgst_entries.iter().enumerate() {
                                    if entry.layer == self.selected_layer && entry.is_valid() {
                                        entry_map.insert(
                                            (
                                                entry.grid_y_position as u32,
                                                entry.grid_x_position as u32,
                                            ),
                                            index,
                                        );
                                    }
                                }
                                // println!("{:#?}", bgst_file);

                                // render by y coordinate

                                for y in 0..bgst_file.grid_height {
                                    for x in 0..bgst_file.grid_width {
                                        let coordinate = (y, x);

                                        if let Some(index) = entry_map.get(&coordinate) {
                                            // render filled square
                                            if let Some(texture_handle) =
                                                self.bgst_renderer.get_texture_handle(*index)
                                            {
                                                // resize image because it's much too big

                                                let image = egui::Image::new(texture_handle)
                                                    .fit_to_exact_size(image_render_size);

                                                let image_button =
                                                    egui::ImageButton::new(image).frame(false); // disable frame, it's distracting

                                                let resp = ui.add(image_button);

                                                if resp.clicked() {
                                                    self.selected_tile =
                                                        Some(TileSelection::Entry(*index));
                                                }

                                                if let Some(tile) = &self.selected_tile
                                                    && *tile == TileSelection::Entry(*index)
                                                {
                                                    ui.painter_at(resp.rect).rect_filled(
                                                        resp.rect,
                                                        0.0,
                                                        egui::Color32::from_rgba_unmultiplied(
                                                            0xFF, 0xFF, 0xFF, 0x10,
                                                        ),
                                                    );
                                                } else if resp.hovered() {
                                                    ui.painter_at(resp.rect).rect_filled(
                                                        resp.rect,
                                                        0.0,
                                                        egui::Color32::from_rgba_unmultiplied(
                                                            0xFF, 0xFF, 0xFF, 0x5,
                                                        ),
                                                    );
                                                }
                                            }
                                        } else {
                                            // render empty square

                                            let (rect, resp) = ui.allocate_exact_size(
                                                image_render_size,
                                                egui::Sense::click(),
                                            );

                                            if resp.clicked() {
                                                self.selected_tile =
                                                    Some(TileSelection::Empty((y, x)));
                                            }

                                            if let Some(tile) = &self.selected_tile
                                                && *tile == TileSelection::Empty((y, x))
                                            {
                                                ui.painter_at(rect).rect(
                                                    rect,
                                                    0.0,
                                                    egui::Color32::from_rgba_unmultiplied(
                                                        0xFF, 0xFF, 0xFF, 0x1,
                                                    ),
                                                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                                                );
                                            } else if resp.hovered() {
                                                ui.painter_at(rect).rect(
                                                    rect,
                                                    0.0,
                                                    egui::Color32::from_rgba_unmultiplied(
                                                        0xFF, 0xFF, 0xFF, 0x1,
                                                    ),
                                                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                                                );
                                            }
                                        }
                                    }

                                    ui.end_row();
                                }
                            }
                        });

                    // image list
                    egui::Area::new(egui::Id::from("be_image_list"))
                        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .inner_margin(egui::Vec2::splat(8.0))
                                .show(ui, |ui| {
                                    ui.collapsing("Image list", |ui| {
                                        self.display_image_list(ui);
                                    });
                                });
                        });
                }); // scroll area
        });
    }

    pub fn handle_selected_tile(&mut self, ui: &mut egui::Ui) {
        if self.selected_tile.is_none() {
            return;
        }

        let mut refresh = false;
        let mut image_removed = false;

        match self.selected_tile.clone().unwrap() {
            TileSelection::Entry(entry_index) => {
                ui.horizontal(|ui| {
                    // image manip options

                    ui.vertical(|ui| {
                        ui.label(format!("Entry Index: {}", entry_index));
                        ui.checkbox(
                            &mut self.bgst_renderer.bgst_file.as_mut().unwrap().bgst_entries
                                [entry_index]
                                .enabled,
                            "Enabled?",
                        );
                        ui.horizontal(|ui| {
                            ui.label("Unknown @ 0xC");
                            ui.add(
                                egui::DragValue::new(
                                    &mut self
                                        .bgst_renderer
                                        .bgst_file
                                        .as_mut()
                                        .unwrap()
                                        .bgst_entries[entry_index]
                                        ._unk_c,
                                )
                                .speed(1)
                                .range(i16::MIN..=i16::MAX),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Unknown @ 0xE");
                            ui.add(
                                egui::DragValue::new(
                                    &mut self
                                        .bgst_renderer
                                        .bgst_file
                                        .as_mut()
                                        .unwrap()
                                        .bgst_entries[entry_index]
                                        ._unk_e,
                                )
                                .speed(1)
                                .range(i16::MIN..=i16::MAX),
                            );
                        });

                        // main image
                        if ui.button("Replace Image").clicked() {
                            let bgst_file = self.bgst_renderer.bgst_file.as_mut().unwrap();
                            let entry = &bgst_file.bgst_entries[entry_index];
                            let _ = bgst_file.replace_image(
                                Some(entry.main_image_index as usize),
                                gctex::TextureFormat::CMPR,
                            );

                            refresh = true;
                        }

                        if ui.button("Remove Image").clicked() {
                            let bgst_file = self.bgst_renderer.bgst_file.as_mut().unwrap();
                            bgst_file.remove_entry(entry_index);
                            self.selected_tile = None;

                            refresh = true;
                            image_removed = true;

                            return; // from the closure
                        }

                        if ui.button("Export Image").clicked() {
                            let bgst_file = self.bgst_renderer.bgst_file.as_ref().unwrap();

                            let _ = bgst_file.export_image(
                                bgst_file.bgst_entries[entry_index].main_image_index as usize,
                                gctex::TextureFormat::CMPR,
                            );
                        }

                        // mask image
                        let bgst_file = self.bgst_renderer.bgst_file.as_mut().unwrap();

                        if bgst_file.bgst_entries[entry_index].is_masked() {
                            if ui.button("Replace Mask").clicked() {
                                let mask_image_index =
                                    bgst_file.bgst_entries[entry_index].mask_image_index;
                                let _ = bgst_file.replace_image(
                                    Some(mask_image_index as usize),
                                    gctex::TextureFormat::I4,
                                );

                                refresh = true;
                            }

                            if ui.button("Remove Mask").clicked()
                                && bgst_file.remove_entry_mask(entry_index)
                            {
                                refresh = true;
                                image_removed = true;

                                return; // from the closure
                            }

                            if ui.button("Export Mask").clicked() {
                                let _ = bgst_file.export_image(
                                    bgst_file.bgst_entries[entry_index].mask_image_index as usize,
                                    gctex::TextureFormat::I4,
                                );
                            }
                        } else if ui.button("Add Mask").clicked()
                            && let Ok(image_index) = bgst_file.add_image(gctex::TextureFormat::I4)
                        {
                            bgst_file.bgst_entries[entry_index].mask_image_index =
                                image_index as i16;

                            refresh = true;
                        }

                        let image_count = self
                            .bgst_renderer
                            .bgst_file
                            .as_ref()
                            .unwrap()
                            .compressed_images
                            .len();

                        let entry =
                            &mut self.bgst_renderer.bgst_file.as_mut().unwrap().bgst_entries
                                [entry_index];

                        let limit = (image_count - 1) as i16;

                        // manual index assignment

                        let mut changed = false;

                        let image_index = &mut entry.main_image_index;

                        ui.horizontal(|ui| {
                            ui.label("Image Index");
                            changed = ui
                                .add(egui::DragValue::new(image_index).speed(1).range(0..=limit))
                                .changed();
                        });

                        // egui clamps the value when the drag value is dragged
                        // but it doesn't account for arrow key input,
                        // so it needs to be clamped further just in case

                        if changed {
                            *image_index = (*image_index).clamp(0, limit);
                        }

                        let masked = entry.is_masked();

                        let image_index = &mut entry.mask_image_index;

                        if masked {
                            ui.horizontal(|ui| {
                                ui.label("Mask Index");
                                changed = ui
                                    .add(
                                        egui::DragValue::new(image_index).speed(1).range(0..=limit),
                                    )
                                    .changed();
                            });

                            if changed {
                                *image_index = (*image_index).clamp(0, limit);

                                refresh = true; // maybe manually refreshing should be an option?
                            }
                        }
                    });

                    // display image(s)
                    if !image_removed && !refresh {
                        let image_render_size = egui::Vec2::splat(100.0);

                        let bgst_file = self.bgst_renderer.bgst_file.as_ref().unwrap();
                        let entry = &bgst_file.bgst_entries[entry_index];

                        // show the main image

                        let main_image_texture_handle = &self.bgst_renderer.decoded_image_handles
                            [entry.main_image_index as usize];
                        ui.add(
                            egui::Image::new(main_image_texture_handle)
                                .fit_to_exact_size(image_render_size),
                        );

                        if entry.is_masked() {
                            // show the mask
                            let mask_image_texture_handle =
                                &self.bgst_renderer.decoded_image_handles
                                    [entry.mask_image_index as usize];

                            ui.add(
                                egui::Image::new(mask_image_texture_handle)
                                    .fit_to_exact_size(image_render_size),
                            );
                        }
                    }
                });
            }

            TileSelection::Empty((y, x)) => {
                if ui.button("Add Image").clicked() {
                    // create new entry
                    refresh = self
                        .bgst_renderer
                        .bgst_file
                        .as_mut()
                        .unwrap()
                        .create_entry(self.selected_layer, (x as i16, y as i16))
                        .is_ok();
                }
            }
        }

        if refresh {
            let _ = self.bgst_renderer.cache_textures(ui.ctx());
        }
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
            for (index, image_handle) in self.bgst_renderer.decoded_image_handles.iter().enumerate()
            {
                body.row(32.0, |mut row| {
                    // image column
                    row.col(|ui| {
                        ui.add(egui::Image::new(image_handle).max_size(egui::Vec2::splat(100.0)));
                    });
                    // index column
                    row.col(|ui| {
                        ui.label(format!("Index {index}"));
                    });
                });
            }
        });
    }
}
