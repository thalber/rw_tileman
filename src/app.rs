use log;

use crate::{
    lingo_de::{self, DeserError},
    lingo_ser,
    utl::*,
    *
};
#[derive(Debug, Clone)]
pub enum AppScheduledAction {
    None,
    Reload,
    MoveCategory(usize, i32),
    DisplayMessage {
        icon: msgbox::IconType,
        title: String,
        text: String,
    }
}
#[derive(Debug)]
pub enum AppError {
    IOError(String),
    Todo,
}
pub struct TilemanApp {
    path_selection: String, //necessary duplicate because egui wants unicode strings
    search_selection: String,
    selected_tile: Option<(usize, usize)>,
    selected_tile_cache: Option<(usize, usize)>,
    preview_cache: Option<PreviewCache>,
    preview_scale: f32,
    init: Option<TileInit>,
    scheduled_action: AppScheduledAction,
    config: AppPersistentConfig,
    pub lhandle: flexi_logger::LoggerHandle,
}

#[derive(Clone)]
pub struct PreviewCache {
    specs: egui::TextureHandle,
    specs2: Option<egui::TextureHandle>,
}

impl TilemanApp {
    pub fn new(
        _cc: &eframe::CreationContext,
        config: AppPersistentConfig,
        lhandle: flexi_logger::LoggerHandle
    ) -> Result<Self, AppError> {
        let init = None;
        let maybe_init = Self::load_data(config.root_path.clone());
        
        let mut tileman_app = Self {
            selected_tile: Default::default(),
            selected_tile_cache: None,
            init,
            preview_cache: None,
            path_selection: config.root_path.to_string_lossy().into_owned(),
            preview_scale: 20f32,
            scheduled_action: AppScheduledAction::None,
            config,
            search_selection: String::new(),
            lhandle,
        };

        tileman_app.apply_loaded_data(maybe_init);
        Ok(tileman_app)
    }
    fn load_data(root: std::path::PathBuf) -> Result<(TileInit, DeserErrorReports), AppError> {
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
                log::error!(
                    "Errors encountered when reading data (ignored on apply) : {errors:#?}\n"
                );
            }
            Err(err) => {
                self.init = None;
                log::error!("Could not load data at all {err:?}");
            }
        };
    }

    fn clear_selection_and_cache(&mut self) {
        self.selected_tile = None;
        self.selected_tile_cache = None;
        self.preview_cache = None;
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
            log::error!("error saving config to disk: {err}")
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
        egui::TopBottomPanel::top("select_path").show(ctx, |ui| {
            ui.label("Path to init");
            let text_input_response = ui.text_edit_singleline(&mut self.path_selection);
            if text_input_response
                .on_hover_text_at_pointer("Enter (copy&paste) path to your editor's tile directory")
                .changed()
            {
                let root = std::path::PathBuf::from(self.path_selection.clone());
                self.apply_loaded_data(Self::load_data(root.clone()));
                self.config.root_path = root;
            }
        });
        let output_path = &mut self.config.output_path;
        let selected_tile = &mut self.selected_tile;
        let selected_tile_cache = &mut self.selected_tile_cache;
        let maybe_preview_cache = &mut self.preview_cache;
        let preview_scale = &mut self.preview_scale;
        //let reload_scheduled = &mut self.reload_scheduled;
        let scheduled_action = &mut self.scheduled_action;
        let search_selection = &mut self.search_selection;
        match &mut self.init {
            Some(init) => {
                //draw action buttons
                egui::TopBottomPanel::top("action_buttons").show(ctx, |ui| {
                    draw_toolbox(ctx, ui, init, preview_scale, scheduled_action, output_path)
                });
                //draw tile list
                egui::SidePanel::right("tile_list").show(ctx, |ui| {
                    draw_tiles_panel(
                        ctx,
                        ui,
                        init,
                        selected_tile,
                        selected_tile_cache,
                        scheduled_action,
                        search_selection,
                    );
                    //ui.set_width(width)
                });
                //draw central panel
                egui::CentralPanel::default().show(ctx, |ui| {
                    draw_central_panel(
                        ctx,
                        ui,
                        selected_tile,
                        selected_tile_cache,
                        init,
                        maybe_preview_cache,
                        preview_scale,
                    );
                });
            }
            None => {
                self.selected_tile_cache = None;
                self.preview_cache = None;
            }
        }
        self.selected_tile_cache = self.selected_tile.clone();

        match self.scheduled_action.clone() {
            AppScheduledAction::None => {}
            AppScheduledAction::Reload => {
                self.apply_loaded_data(Self::load_data(std::path::PathBuf::from(
                    self.path_selection.clone(),
                )));
                self.clear_selection_and_cache();
            }
            AppScheduledAction::MoveCategory(old_index, by) => {
                if let Some(init) = &mut self.init {
                    let new_index = (old_index as i32 + by).max(0) as usize;
                    log::debug!("moving {old_index} by {by}");
                    init.categories.get_mut(old_index).and_then(|a| {
                        a.index = new_index;
                        Some(a)
                    });
                    init.categories.get_mut(new_index).and_then(|a| {
                        a.index = old_index;
                        Some(a)
                    });

                    init.sort_and_normalize_categories();
                }
                self.clear_selection_and_cache();
            }
            AppScheduledAction::DisplayMessage { icon, title, text } => {
                
                let msgbox_res = msgbox::create(&title, &text, icon);
                log::info!("{msgbox_res:?}")
            },
        }
        self.scheduled_action = AppScheduledAction::None;
        // if self.reload_scheduled {
        //     self.apply_loaded_data(Self::load_data(std::path::PathBuf::from(
        //         self.path_selection.clone(),
        //     )))
        // }
        // self.reload_scheduled = false;
    }
}

fn draw_central_panel(
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
            if let Some(Some(item)) = init
                .categories
                .get_mut(*category_index)
                .map(|cat| cat.tiles.get_mut(*item_index))
            {
                draw_tile_details(
                    ctx,
                    ui,
                    preview_scale,
                    item,
                    maybe_preview_cache,
                    changed_selection,
                );
            }
        }
        _ => (),
    };
}

fn draw_tile_details(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preview_scale: &mut f32,
    item: &mut TileInfo,
    maybe_preview_cache: &mut Option<PreviewCache>,
    changed_selection: bool,
) {
    ui.heading(item.name.clone());
    let mut default_string = String::new();
    egui::ScrollArea::vertical()
        .id_source("edit_tags_section")
        .show(ui, |ui| {
            let mut maybe_remove = None;
            for tag_index in indices(&item.tags) {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(
                        item.tags.get_mut(tag_index).unwrap_or(&mut default_string),
                    );
                    if ui.button("remove").clicked() {
                        maybe_remove = Some(tag_index);
                        log::debug!("removing tag {tag_index} from {}", item.name.clone())
                    }
                });
            }
            if (ui.button("Add tag")).clicked() {
                item.tags.push(String::new());
                log::debug!("adding tag to {}", item.name.clone())
            }
            if let Some(remove) = maybe_remove {
                item.tags.remove(remove);
            }

            egui::ScrollArea::horizontal()
                .id_source("preview_specs_section")
                .show(ui, |ui| {
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
                    let mut maybe_thandle_s2 =
                        match (maybe_preview_cache.clone(), changed_selection) {
                            (Some(thandle), false) => thandle.specs2,
                            _ => None,
                        };
                    if item.specs2.is_some() {
                        ui.heading("specs2");
                        maybe_thandle_s2 = maybe_thandle_s2
                            .or_else(|| Some(create_specs_texture(ctx, item, true)));
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
                });
        });
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
    log::info!(
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
    scheduled_action: &mut AppScheduledAction,
    search_selection: &mut String,
) {
    ui.label("search");
    ui.text_edit_singleline(search_selection)
        .on_hover_text_at_pointer("Search tiles");
    ui.heading("tiles");
    egui::ScrollArea::vertical().show(ui, |ui| {
        for category_index in indices(&init.categories) {
            let category = &mut init.categories[category_index];
            egui::CollapsingHeader::new(category.name.as_str())
                .show(ui, |ui| {
                    list_tile_category(
                        ctx,
                        ui,
                        &mut init.root,
                        category,
                        selected_tile,
                        selected_tile_cache,
                        category_index,
                        scheduled_action,
                        search_selection,
                    );
                })
                .header_response
                .on_hover_text_at_pointer(match category.subfolder {
                    Some(_) => "A subfolder",
                    None => "Exists in main init only",
                });
        }
    });
}

fn list_tile_category(
    _ctx: &egui::Context,
    ui: &mut egui::Ui,
    _root: &mut std::path::PathBuf,
    category: &mut crate::TileCategory,
    selected_tile: &mut Option<(usize, usize)>,
    _selected_tile_cache: &mut Option<(usize, usize)>,
    category_index: usize,
    scheduled_action: &mut AppScheduledAction,
    search_selection: &mut String,
) {
    let is_folder = category.subfolder.is_some();
    //ui.text_edit_singleline(&mut category.name);
    if is_folder {
        ui.checkbox(&mut category.enabled, "Enable category");
    }
    //format!("{}_change", category.name.clone()),
    egui::ComboBox::from_label("Change")
        .selected_text(format!("{:?}", category.scheduled_change))
        .show_ui(ui, |ui| {
            macro_rules! add_choice {
                ($item:ident) => {
                    ui.selectable_value(
                        &mut category.scheduled_change,
                        TileCategoryChange::$item,
                        stringify!($item),
                    );
                };
            }
            add_choice!(None);
            add_choice!(MoveFromSubfolder);
            add_choice!(MoveToSubfolder);
            add_choice!(Delete);
        });
    ui.horizontal(|ui| {
        if ui
            .button("[ ^ ]")
            .on_hover_text_at_pointer("Move category up")
            .clicked()
        {
            *scheduled_action = AppScheduledAction::MoveCategory(category_index, -1);
        }
        if ui
            .button("[ v ]")
            .on_hover_text("Move category down")
            .clicked()
        {
            *scheduled_action = AppScheduledAction::MoveCategory(category_index, 1);
        }
    });
    for item_index in indices(&category.tiles) {
        let item = &mut category.tiles[item_index];
        if !tile_info_matches_search(item, search_selection) {
            continue;
        }
        ui.horizontal(|ui| {
            if is_folder {
                ui.checkbox(&mut item.active, "");
            }
            if ui.button(item.name.as_str()).clicked() {
                *selected_tile = Some((category_index, item_index));
            };
        });
    }
}

fn tile_info_matches_search(item: &TileInfo, search_selection: &String) -> bool {
    if search_selection.is_empty() {
        return true;
    }
    name_matches_search(&item.name, search_selection)
        || item
            .tags
            .iter()
            .any(|tag| name_matches_search(tag, search_selection))
}

fn draw_toolbox(
    _ctx: &egui::Context,
    ui: &mut egui::Ui,
    init: &mut TileInit,
    preview_scale: &mut f32,
    scheduled_action: &mut AppScheduledAction,
    output_path: &mut std::path::PathBuf,
) {
    ui.horizontal(|ui| {
        if ui
            .button("save")
            .on_hover_text_at_pointer("Write main and subfolder inits to disk (creates a backup)")
            .clicked()
        {
            
            if let Err((err, _)) = lingo_ser::rewrite_init(&init, output_path.clone()) {
                *scheduled_action = AppScheduledAction::DisplayMessage 
                {
                    icon: msgbox::IconType::Error,
                    title: String::from("Error saving inits"),
                    text: format!("failed to save inits to disk due to the following error: {err:?}. details in tileman.log") 
                };
                //format!("failed to save inits to disk due to the following error: {err:?}. details in tileman.log")
            }
            else {
                *scheduled_action = AppScheduledAction::Reload;
                //log::info!("saved with result {:#?}", result)
            }
            
        };
        if (ui.button("reload"))
            .on_hover_text_at_pointer("Reload inits from disk")
            .clicked()
        {
            *scheduled_action = AppScheduledAction::Reload;
        }
        ui.add(egui::Slider::new(preview_scale, 5f32..=40f32))
            .on_hover_text_at_pointer("Select tile preview scale");
        if ui
            .button("all2sub")
            .on_hover_text_at_pointer("Move all categories from main init to subfolders")
            .clicked()
        {
            for cat in init.categories.iter_mut() {
                if cat.subfolder.is_none() {
                    cat.scheduled_change = TileCategoryChange::MoveToSubfolder;
                }
            }
        }
        if ui
            .button("all2main")
            .on_hover_text_at_pointer("Move all categories from subfolders to main init")
            .clicked()
        {
            for cat in init.categories.iter_mut() {
                if cat.subfolder.is_some() {
                    cat.scheduled_change = TileCategoryChange::MoveFromSubfolder;
                }
            }
        }
        if ui.button("test popup").clicked() {
            *scheduled_action = AppScheduledAction::DisplayMessage 
            { 
                icon: msgbox::IconType::None, 
                title: String::from("que"), 
                text: String::from("guh")
            }
        }
    });
}
