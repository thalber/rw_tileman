use std::path::PathBuf;

use egui::{epaint::ImageDelta, CollapsingHeader, Color32, ImageData};

use crate::{
    lingo_de::{self, DeserError},
    lingo_ser,
    utl::*,
    DeserErrorReports, TileInfo, TileInit,
};

use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AppPersistentConfig {
    pub root_path: PathBuf,
    pub output_path: PathBuf,
}
#[derive(Debug)]
pub enum AppError {
    IOError(String),
    Todo,
}
pub struct TilemanApp {
    path_selection: String, //necessary because egui wants unicode strings
    //output_path: PathBuf,
    selected_tile: Option<(usize, usize)>,
    selected_tile_cache: Option<(usize, usize)>,
    preview_cache: Option<PreviewCache>,
    preview_scale: f32,
    init: Option<TileInit>,
    reload_scheduled: bool,
    config: AppPersistentConfig,
}

#[derive(Clone)]
pub struct PreviewCache {
    //position: (usize, usize),
    specs: egui::TextureHandle,
    specs2: Option<egui::TextureHandle>,
}

impl TilemanApp {
    pub fn new(
        _cc: &eframe::CreationContext,
        config: AppPersistentConfig,
    ) -> Result<Self, AppError> {
        let init = None;
        let maybe_init = Self::load_data(config.root_path.clone());
        let mut tileman_app = Self {
            selected_tile: Default::default(),
            selected_tile_cache: None,
            init,
            preview_cache: None,
            //output_path: out.clone(),
            path_selection: config.root_path.to_string_lossy().into_owned(),
            preview_scale: 20f32,
            reload_scheduled: false,
            config,
        };
        tileman_app.apply_loaded_data(maybe_init);
        Ok(tileman_app)
    }
    fn load_data(root: PathBuf) -> Result<(TileInit, DeserErrorReports), AppError> {
        let mut errors = Vec::new();
        let additional_categories = lingo_de::collect_categories_from_subfolders(root.clone())
            .unwrap_or(Vec::new())
            .into_iter()
            .map(|(category, newerrors)| {
                for newerror in newerrors {
                    errors.push(newerror)
                }
                category
            })
            .collect();
        let text = std::fs::read_to_string(root.join("init.txt"));
        match text {
            Ok(text) => {
                let init = lingo_de::parse_tile_init(text, additional_categories, root)?;
                return Ok((init, errors));
            }
            Err(err) => Err(AppError::IOError(format!("{:?}", err))),
        }
    }

    fn apply_loaded_data(
        &mut self,
        maybe_init: Result<(TileInit, Vec<(String, DeserError)>), AppError>,
    ) {
        match maybe_init {
            Ok((actual_init, errors)) => {
                //init = Some(actual_init);
                self.init = Some(actual_init);
                std::fs::write(
                    self.config.output_path.join("tileman_errors.txt"),
                    format!("{:#?}", errors),
                )
                .expect("could not write errors");
            }
            Err(err) => {
                self.init = None;
                std::fs::write(
                    self.config.output_path.join("tileman_errors.txt"),
                    format!("{:?}", err),
                )
                .expect("could not write errors")
            }
        };
    }
}

impl eframe::App for TilemanApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn on_close_event(&mut self) -> bool {
        if let Err(err) = std::fs::write(
            std::env::current_dir()
                .expect("could not get wd")
                .join("tileman_config.json"),
            serde_json::ser::to_string(&self.config).expect("could not serialize config"),
        ) {
            println!("{}", err);
        }

        true
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        egui::Vec2::INFINITY
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()

        // _visuals.window_fill() would also be a natural choice
    }

    fn persist_native_window(&self) -> bool {
        true
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ctx.tessellation_options_mut(|op| {
        //     op.feathering = false;
        //     op.feathering_size_in_pixels = 0f32;
        // });

        egui::TopBottomPanel::top("select_path").show(ctx, |ui| {
            ui.label("Path to init");
            let text_input_response = ui.text_edit_singleline(&mut self.path_selection);
            if text_input_response
                .on_hover_text("Enter (copy&paste) path to your editor's tile directory")
                .changed()
            {
                let root = PathBuf::from(self.path_selection.clone());
                self.apply_loaded_data(Self::load_data(root.clone()));
                self.config.root_path = root;
            }
        });
        let output_path = &mut self.config.output_path;
        let selected_tile = &mut self.selected_tile;
        let selected_tile_cache = &mut self.selected_tile_cache;
        let maybe_preview_cache = &mut self.preview_cache;
        let preview_scale = &mut self.preview_scale;
        let reload_scheduled = &mut self.reload_scheduled;
        match &mut self.init {
            Some(init) => {
                //draw action buttons
                egui::TopBottomPanel::top("action_buttons").show(ctx, |ui| {
                    draw_toolbox(ctx, ui, init, preview_scale, reload_scheduled, output_path)
                });
                //draw tile list
                egui::SidePanel::right("tile_list").show(ctx, |ui| {
                    draw_tiles_panel(ctx, ui, init, selected_tile, selected_tile_cache)
                });
                //draw central panel
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        draw_specs_previews(
                            ctx,
                            ui,
                            selected_tile,
                            selected_tile_cache,
                            init,
                            maybe_preview_cache,
                            preview_scale,
                        );
                    })
                });
            }
            None => {
                self.selected_tile_cache = None;
                self.preview_cache = None;
            }
        }
        self.selected_tile_cache = self.selected_tile.clone();
        if self.reload_scheduled {
            self.apply_loaded_data(Self::load_data(PathBuf::from(self.path_selection.clone())))
        }
        self.reload_scheduled = false;
    }
}

fn draw_specs_previews(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    selected_tile: &mut Option<(usize, usize)>,
    selected_tile_cache: &mut Option<(usize, usize)>,
    init: &mut TileInit,
    maybe_preview_cache: &mut Option<PreviewCache>,
    preview_scale: &mut f32,
) {
    //let (old_categpory_index, old_item_index) = selected_tile_cache.unwrap_or((usize::MAX, usize::MAX));
    let changed_selection = *selected_tile != *selected_tile_cache;
    match selected_tile {
        Some((category_index, item_index)) => {
            let maaaybe_item = init
                .categories
                .get(*category_index)
                .map(|cat| cat.tiles.get(*item_index));
            if let Some(Some(item)) = maaaybe_item {
                let (size_x, size_y) = (
                    *item.size.get(0).unwrap_or(&1) as usize,
                    *item.size.get(1).unwrap_or(&1) as usize,
                );
                ui.heading("specs1");
                let maybe_thandle_s1 = match (maybe_preview_cache.clone(), changed_selection) {
                    (Some(thandle), false) => Some(thandle.specs),
                    _ => None,
                };
                let thandle_s1 =
                    maybe_thandle_s1.unwrap_or_else(|| create_specs_texture(ctx, item, false));
                ui.image(
                    thandle_s1.id(),
                    egui::vec2(
                        size_x as f32 * *preview_scale,
                        size_y as f32 * *preview_scale,
                    ),
                );

                let mut maybe_thandle_s2 = match (maybe_preview_cache.clone(), changed_selection) {
                    (Some(thandle), false) => thandle.specs2,
                    _ => None,
                };

                if item.specs2.is_some() {
                    ui.heading("specs2");
                    maybe_thandle_s2 =
                        maybe_thandle_s2.or_else(|| Some(create_specs_texture(ctx, item, true)));
                    ui.image(
                        maybe_thandle_s2
                            .clone()
                            .expect("this should not happen: specs2 draw")
                            .id(),
                        egui::vec2(
                            size_x as f32 * *preview_scale,
                            size_y as f32 * *preview_scale,
                        ),
                    );
                }

                *maybe_preview_cache = Some(PreviewCache {
                    specs: thandle_s1,
                    specs2: maybe_thandle_s2,
                })
            }
        }
        _ => (),
    };
}

fn create_specs_texture(
    ctx: &egui::Context,
    item: &TileInfo,
    take_specs2: bool,
) -> egui::TextureHandle {
    let postfix = match take_specs2 {
        true => "s1",
        false => "s2",
    };
    // let (size_x, size_y) = (
    //     *item.size.get(0).unwrap_or(&0) as usize,
    //     *item.size.get(1).unwrap_or(&0) as usize,
    // );
    let cells = item.display_cells(take_specs2);
    let dim = cells.extents();
    let size_x = *dim.get(0).unwrap_or(&0);
    let size_y = *dim.get(1).unwrap_or(&0);

    let mut image = egui::ColorImage::new([size_x, size_y], egui::Color32::from_rgb(255, 255, 255));
    for x in 0..(size_x) {
        for y in 0..(size_y) {
            let index_to_take = y * size_x + x;
            let display_color = cells[[x, y]].display_color();
            image.pixels[index_to_take] =
                egui::Color32::from_rgb(display_color[0], display_color[1], display_color[2]);
        }
    }
    let name = format!("{}-{}", item.name.clone(), postfix);
    println!(
        "creating a new texture for {} - {}",
        item.name.clone(),
        name.clone()
    );
    let mut options = egui::TextureOptions::default();
    options.magnification = egui::TextureFilter::Nearest;
    ctx.load_texture(name, egui::ImageData::Color(image), options)
}

fn draw_tiles_panel(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    init: &mut TileInit,
    selected_tile: &mut Option<(usize, usize)>,
    selected_tile_cache: &mut Option<(usize, usize)>,
) {
    ui.heading("tiles");
    egui::ScrollArea::vertical().show(ui, |ui| {
        for category_index in indices(&init.categories) {
            let category = &mut init.categories[category_index];
            CollapsingHeader::new(category.name.as_str())
                .show(ui, |ui| {
                    list_tile_category(
                        ctx,
                        ui,
                        category,
                        selected_tile,
                        selected_tile_cache,
                        category_index,
                    );
                })
                .header_response
                .on_hover_text(match category.subfolder {
                    Some(_) => "A subfolder",
                    None => "Exists in main init only",
                });
        }
    });
}

fn list_tile_category(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    category: &mut crate::TileCategory,
    selected_tile: &mut Option<(usize, usize)>,
    selected_tile_cache: &mut Option<(usize, usize)>,
    category_index: usize,
) {
    for item_index in indices(&category.tiles) {
        let item = &mut category.tiles[item_index];
        ui.horizontal(|ui| {
            if category.subfolder.is_some() {
                ui.checkbox(&mut item.active, "");
            }
            if ui.button(item.name.as_str()).clicked() {
                println!("{}", item.name);
                *selected_tile = Some((category_index, item_index));
            };
        });
    }
}

fn draw_toolbox(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    init: &mut TileInit,
    preview_scale: &mut f32,
    reload_scheduled: &mut bool,
    output_path: &mut PathBuf,
) {
    ui.horizontal(|ui| {
        if ui
            .button("save inits")
            .on_hover_text("Write main and subfolder inits to disk (creates a backup)")
            .clicked()
        {
            let result = lingo_ser::rewrite_init(&init, output_path.clone());
            std::fs::write(
                output_path.join("write_report.txt"),
                format!("{:#?}", result),
            )
            .expect("Could not write errors");
        };
        if (ui.button("reload"))
            .on_hover_text("Reload inits from disk")
            .clicked()
        {
            *reload_scheduled = true;
        }
        ui.add(egui::Slider::new(preview_scale, 10f32..=40f32))
            .on_hover_text("Select tile preview scale");
    });
}
