use std::path::PathBuf;

use eframe::App;
use egui::CollapsingHeader;

use crate::{
    lingo_de::{self, DeserError},
    lingo_ser,
    utl::*,
    DeserErrorReports, TileInfo, TileInit,
};
#[derive(Debug)]
pub enum AppError {
    IOError(String),
    Todo,
}
pub struct TilemanApp {
    path_selection: String,
    output_path: PathBuf,
    selected_tile: Option<(usize, usize)>,
    init: Option<TileInit>,
    dumped_errors: bool,
}

impl TilemanApp {
    pub fn new(
        _cc: &eframe::CreationContext,
        root: PathBuf,
        out: PathBuf,
    ) -> Result<Self, AppError> {
        let init = None;
        let maybe_init = Self::load_data(root.clone());
        let mut tileman_app = Self {
            selected_tile: Default::default(),
            init,
            dumped_errors: false,
            output_path: out.clone(),
            path_selection: root.to_string_lossy().into_owned(),
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
                    self.output_path.join("tileman_errors.txt"),
                    format!("{:#?}", errors),
                )
                .expect("could not write errors");
            }
            Err(err) => {
                self.init = None;
                std::fs::write(
                    self.output_path.join("tileman_errors.txt"),
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
        egui::TopBottomPanel::top("select_path").show(ctx, |ui| {
            if ui.text_edit_singleline(&mut self.path_selection).changed() {
                self.apply_loaded_data(Self::load_data(PathBuf::from(self.path_selection.clone())));
            }
        });

        match &mut self.init {
            Some(init) => {
                egui::TopBottomPanel::top("action_buttons").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("save inits").clicked() {
                            let result = lingo_ser::rewrite_init(&init);
                            std::fs::write(
                                self.output_path.join("write_report.txt"),
                                format!("{:#?}", result),
                            )
                            .expect("Could not write errors");
                        };
                    })
                });
                egui::SidePanel::right("right_panel").show(ctx, |ui| {
                    ui.heading("tiles");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for category_index in indices(&init.categories) {
                            let category = &mut init.categories[category_index];
                            CollapsingHeader::new(category.name.as_str()).show(ui, |ui| {
                                for item_index in indices(&category.tiles) {
                                    let item = &mut category.tiles[item_index];
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut item.active, "");
                                        if ui.button(item.name.as_str()).clicked() {
                                            println!("{}", item.name);
                                            self.selected_tile = Some((category_index, item_index));
                                        };
                                    });
                                }
                            });
                        }
                    })
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Path to init");
                    if let Some((category_index, item_index)) = self.selected_tile {
                        let maaaaybe_item = init
                            .categories
                            .get(category_index)
                            .map(|cat| cat.tiles.get(item_index));
                        if let Some(Some(item)) = maaaaybe_item {
                            ui.label(format!("{:?}", item));
                        }
                    }
                });

                if !self.dumped_errors {
                    std::fs::write(
                        init.root.join("tileman_errors.txt"),
                        format!("{:#?}", init.errored_lines),
                    )
                    .expect("could not write results");
                    self.dumped_errors = true;
                }
            }
            None => { },
        }
    }
}
