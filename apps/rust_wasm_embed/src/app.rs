use erust::applicationstate::AppState;

use log::{ info, warn };

///	Egui Tile Edits by h0lybyte and Emilk for base example.

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pane {
	nr: usize,
}

impl std::fmt::Debug for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View").field("nr", &self.nr).finish()
    }
}

impl Pane {
    pub fn with_nr(nr: usize) -> Self {
        Self { nr }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui_tiles::UiResponse {
        let color = egui::epaint::Hsva::new(0.103 * self.nr as f32, 0.5, 0.5, 1.0);
        ui.painter().rect_filled(ui.max_rect(), 0.0, color);
        let dragged = ui
            .allocate_rect(ui.max_rect(), egui::Sense::drag())
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dragged();
        if dragged {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}

struct TreeBehavior {
	simplification_options: egui_tiles::SimplificationOptions,
    tab_bar_height: f32,
    gap_width: f32,
    add_child_to: Option<egui_tiles::TileId>,
}

impl Default for TreeBehavior {
    fn default() -> Self {
        Self {
            simplification_options: Default::default(),
            tab_bar_height: 24.0,
            gap_width: 2.0,
            add_child_to: None,
        }
    }
}

impl TreeBehavior {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            simplification_options,
            tab_bar_height,
            gap_width,
            add_child_to: _,
        } = self;

        egui::Grid::new("behavior_ui")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("All panes must have tabs:");
                ui.checkbox(&mut simplification_options.all_panes_must_have_tabs, "");
                ui.end_row();

                ui.label("Join nested containers:");
                ui.checkbox(
                    &mut simplification_options.join_nested_linear_containers,
                    "",
                );
                ui.end_row();

                ui.label("Tab bar height:");
                ui.add(
                    egui::DragValue::new(tab_bar_height)
                        .clamp_range(0.0..=100.0)
                        .speed(1.0),
                );
                ui.end_row();

                ui.label("Gap width:");
                ui.add(
                    egui::DragValue::new(gap_width)
                        .clamp_range(0.0..=20.0)
                        .speed(1.0),
                );
                ui.end_row();
            });
    }
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        view.ui(ui)
    }

    fn tab_title_for_pane(&mut self, view: &Pane) -> egui::WidgetText {
        format!("View Tab {}", view.nr).into()
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("➕").clicked() {
            self.add_child_to = Some(tile_id);
        }
    }

    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        self.tab_bar_height
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        self.gap_width
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        self.simplification_options
    }
}




///	TODO Migrate State to ERust States

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RustWasmEmbedApp {
	state: AppState, // State
	tree: egui_tiles::Tree<Pane>, // Egui Tiles
	#[serde(skip)]
    behavior: TreeBehavior, // Egui Tiles -> TreeBehavior
}

impl Default for RustWasmEmbedApp {
	fn default() -> Self {
		// init -> Pane, Tabs, via ReRun's Advance @i.

		Self {
			state: AppState::default(),
			tree: create_tree(),
			behavior: Default::default(),
		}
	}
}

impl RustWasmEmbedApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {



		let app = Self {
			state: AppState::load(cc.storage).unwrap_or_else(AppState::new),
			tree: create_tree(),
			behavior: Default::default(),
			
		};

		if app.state.is_dark_mode {
			cc.egui_ctx.set_visuals(egui::Visuals::dark());
		} else {
			cc.egui_ctx.set_visuals(egui::Visuals::light());
		}

		app
	}
}

impl eframe::App for RustWasmEmbedApp {
	/// Called by the frame work to save state before shutdown.

	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.state.save(storage);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		// let url = "https://kbve.com/assets/img/curved-images/wave.jpg".to_string();
		// self.state.start_image_loading(ctx, url);

		if !self.state.is_image_loaded {
			// Call load_image only once
			//let base64_string = "";
			//self.state.load_image(ctx, "https://rareicon.com/assets/images/nextjs-landing-page-banner.png");
			//self.state.load_image_from_base64(ctx, base64_string);
			// Mark the image as being loaded
			self.state.is_image_loaded = true;
		}



		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:
			
			ui.add_space(8.0);
			egui::menu::bar(ui, |ui| {
				// NOTE: no File->Quit on web pages!
				let is_web = cfg!(target_arch = "wasm32");
				if !is_web {
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
					});
				}
				if erust::widgets::dark_mode_widget(ui, &mut self.state) {
					// If state changed, save the updated state
					if let Some(storage) = _frame.storage_mut() {
						self.state.save(storage);
					}
				}
			});
			ui.add_space(8.0);


		});

		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("Side Panel");

			// Tree UI
			if ui.button("Reset").clicked() {
                *self = Default::default();
            }
            self.behavior.ui(ui);

			ui.collapsing("Tree", |ui| {
                ui.style_mut().wrap = Some(false);
                let tree_debug = format!("{:#?}", self.tree);
                ui.monospace(&tree_debug);
            });

			ui.separator();

			ui.collapsing("Active tiles", |ui| {
                let active = self.tree.active_tiles();
                for tile_id in active {
                    use egui_tiles::Behavior as _;
                    let name = self.behavior.tab_title_for_tile(&self.tree.tiles, tile_id);
                    ui.label(format!("{} - {tile_id:?}", name.text()));
                }
            });

            ui.separator();

            if let Some(root) = self.tree.root() {
                tree_ui(ui, &mut self.behavior, &mut self.tree.tiles, root);
            }

            if let Some(parent) = self.behavior.add_child_to.take() {
                let new_child = self.tree.tiles.insert_pane(Pane::with_nr(100));
                if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                    self.tree.tiles.get_mut(parent)
                {
                    tabs.add_child(new_child);
                    tabs.set_active(new_child);
                }
            }

			ui.separator();


			ui.horizontal(|ui| {
				ui.label("Adjust value: ");
				if ui.button("Increment").clicked() {
					self.state.value += 1.0;
				}
			});

			ui.add_space(5.0);

			// Add more widgets here as needed
			if ui.button("Save State").clicked() {
				// Check if storage is available and get a mutable reference
				if let Some(storage) = _frame.storage_mut() {
					// Now storage is a mutable reference
					self.state.save(storage);
				}
			}

			// Bottom Up UI Approach

			ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
				//  Dark / Light
				
				
				ui.add_space(8.0);

				if erust::widgets::dark_mode_widget(ui, &mut self.state) {
					// If state changed, save the updated state
					if let Some(storage) = _frame.storage_mut() {
						self.state.save(storage);
					}
				}
				

			});
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			// let size = ui.available_size();
			// let rect = egui::Rect::from_min_size(ui.min_rect().min, size);

			// if let Some(texture_handle) = self.state.image_texture.lock().unwrap().as_ref() {
			//     // Paint the image directly onto the canvas as a background
			//     ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE); // Background color (optional)
			//     ui.painter().add(egui::Shape::image(
			//         texture_handle.id(),
			//         rect,
			//         rect, // Use the same rect for UV to draw the whole image
			//         egui::Color32::BLACK,
			//     ));
			// }

			// let size = ui.available_size();
			// let (_, painter) = ui.allocate_painter(size, egui::Sense::hover());

			// if let Some(texture_handle) = self.state.image_texture.lock().unwrap().as_ref() {
			//     let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), size);
			//     painter.rect_filled(rect, 0.0, egui::Color32::WHITE); // Optional background color
			//     painter.image(texture_handle.id(), rect, rect, egui::Color32::WHITE); // Draw the image
			// }

			//	Header for the Applications
			ui.heading("eRust");

			// The central panel the region left after adding TopPanel's and SidePanel's

			ui.horizontal(|ui| {
				ui.label("Write something: ");
				ui.text_edit_singleline(&mut self.state.label);
			});

			ui.add(
				egui::Slider
					::new(&mut self.state.value, 0.0..=10.0)
					.text("value")
			);
			if ui.button("Increment").clicked() {
				self.state.value += 1.0;
			}

			ui.separator();

			egui::CentralPanel::default().show(ctx, |ui| {
				self.tree.ui(&mut self.behavior, ui);
			});

			
		

			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				erust::ironatom::powered_by_egui_and_eframe(ui);
				egui::warn_if_debug_build(ui);
				ui.add(
					egui::github_link_file!(
						"https://github.com/kbve/kbve/blob/main/",
						"Source code."
					)
				);
			});
		});
	}
}


///	Tree_UI


fn tree_ui(
    ui: &mut egui::Ui,
    behavior: &mut dyn egui_tiles::Behavior<Pane>,
    tiles: &mut egui_tiles::Tiles<Pane>,
    tile_id: egui_tiles::TileId,
) {
    // Get the name BEFORE we remove the tile below!
    let text = format!(
        "{} - {tile_id:?}",
        behavior.tab_title_for_tile(tiles, tile_id).text()
    );

    // Temporarily remove the tile to circumvent the borrowchecker
    let Some(mut tile) = tiles.remove(tile_id) else {
        log::warn!("Missing tile {tile_id:?}");
        return;
    };

    let default_open = true;
    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        egui::Id::new((tile_id, "tree")),
        default_open,
    )
    .show_header(ui, |ui| {
        ui.label(text);
        let mut visible = tiles.is_visible(tile_id);
        ui.checkbox(&mut visible, "Visible");
        tiles.set_visible(tile_id, visible);
    })
    .body(|ui| match &mut tile {
        egui_tiles::Tile::Pane(_) => {}
        egui_tiles::Tile::Container(container) => {
            let mut kind = container.kind();
            egui::ComboBox::from_label("Kind")
                .selected_text(format!("{kind:?}"))
                .show_ui(ui, |ui| {
                    for typ in egui_tiles::ContainerKind::ALL {
                        ui.selectable_value(&mut kind, typ, format!("{typ:?}"))
                            .clicked();
                    }
                });
            if kind != container.kind() {
                container.set_kind(kind);
            }

            for &child in container.children() {
                tree_ui(ui, behavior, tiles, child);
            }
        }
    });

    // Put the tile back
    tiles.insert(tile_id, tile);
}


fn create_tree() -> egui_tiles::Tree<Pane> {

	let mut next_view_nr = 0;
	let mut gen_view = || {
		let view = Pane::with_nr(next_view_nr);
		next_view_nr += 1;
		view
	};

	let mut tiles = egui_tiles::Tiles::default();

	let mut tabs = vec![];
	let tab_tile = {
		let children = (0..7).map(|_| tiles.insert_pane(gen_view())).collect();
		tiles.insert_tab_tile(children)
	};
	tabs.push(tab_tile);
	tabs.push({
		let children = (0..7).map(|_| tiles.insert_pane(gen_view())).collect();
		tiles.insert_horizontal_tile(children)
	});
	tabs.push({
		let children = (0..7).map(|_| tiles.insert_pane(gen_view())).collect();
		tiles.insert_vertical_tile(children)
	});
	tabs.push({
		let cells = (0..11).map(|_| tiles.insert_pane(gen_view())).collect();
		tiles.insert_grid_tile(cells)
	});
	tabs.push(tiles.insert_pane(gen_view()));

	let root = tiles.insert_tab_tile(tabs);

	egui_tiles::Tree::new("app_tree", root, tiles)

}