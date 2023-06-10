use crate::{
    lingo_ser, SerErrorReports, TileCategory, TileCategoryChange, TileCell, TileInfo, TileInit,
    CATEGORY_ON_MARKER,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SerError {
    InitBackupFailed,
    IOError { text: String, category: String },
    Todo,
}

pub fn rewrite_init(
    init: &TileInit,
    output_path: std::path::PathBuf,
) -> Result<SerErrorReports, (SerError, SerErrorReports)> {
    let now = std::time::Instant::now();
    let mut main_init_to_write = String::new();
    let mut errors = backup_init_files(init);
    if errors.len() > 0 {
        log::error!(
            "encountered errors during backup: {:#?}",
            (now, errors.clone())
        );
        return Err((SerError::InitBackupFailed, errors));
        //panic!("Could not create init backups!")
    }
    for mut category in init.categories.clone().into_iter() {
        match category.scheduled_change {
            TileCategoryChange::None => {}
            TileCategoryChange::MoveToSubfolder => {
                category.subfolder = Some(init.root.join(category.name.clone()));
            }
            TileCategoryChange::Delete => {}
            _ => {} //TileCategoryChange::MoveFromSubfolder => category.subfolder = None,
        }
        log::debug!(
            "{:?}, {:?}",
            category.scheduled_change.clone(),
            category.subfolder
        );

        let cat_text_noexclude = serialize_category(&category, false)
            .into_iter()
            .fold(String::new(), |sum, new| format!("{sum}\n{new}"));
        let cat_text_exclude = serialize_category(&category, true)
            .into_iter()
            .fold(String::new(), |sum, new| format!("{sum}\n{new}"));
        let (cat_text_for_main, cat_text_for_sub) =
            match (category.enabled, category.scheduled_change.clone()) {
                (_, TileCategoryChange::Delete) => (String::new(), String::new()),
                (true, TileCategoryChange::None) => (cat_text_exclude, cat_text_noexclude),
                (false, TileCategoryChange::None) => (String::new(), cat_text_noexclude),
                (true, TileCategoryChange::MoveToSubfolder) => {
                    (cat_text_exclude, cat_text_noexclude)
                }
                (false, TileCategoryChange::MoveToSubfolder) => (String::new(), cat_text_noexclude),
                (_, TileCategoryChange::MoveFromSubfolder) => (cat_text_noexclude, String::new()),
                //(_, TileCategoryChange::Rename(_)) => todo!(),
                // (false, TileCategoryChange::Rename(_)) => todo!(),
            };
        main_init_to_write.push('\n');
        main_init_to_write.push_str(cat_text_for_main.as_str());

        if let Some(sub) = category.subfolder.clone() {
            let init_path = sub.join("init.txt");
            if !sub.exists() {
                std::fs::create_dir(sub.clone())
                    .expect(format!("could not create dir {:?}", sub.clone()).as_str());
            }
            match category.scheduled_change {
                TileCategoryChange::Delete | TileCategoryChange::MoveFromSubfolder => {
                    if let Err(err) = std::fs::remove_file(init_path.clone()) {
                        errors.push(lingo_ser::SerError::IOError {
                            text: format!("{err:?}"),
                            category: category.name.clone(),
                        });
                    };
                }
                _ => {
                    if let Err(err) = std::fs::write(init_path, cat_text_for_sub) {
                        errors.push(SerError::IOError {
                            text: format!("{err:?}"),
                            category: category.name.clone(),
                        });
                    }
                }
            }
            //copy tile files
            let png_errors = category.tiles.iter().filter_map(|tile| {
                let filename = format!("{}.png", tile.name);
                let png_in_sub = sub.join(filename.clone());
                let png_in_root = init.root.join(filename.clone());
                // let (from, to) = match category.scheduled_move_to_sub {
                //     true => (png_in_root, png_in_sub),
                //     false => (png_in_sub, png_in_root),
                // };
                let (from, to) = match category.scheduled_change {
                    TileCategoryChange::None => (png_in_sub, png_in_root),
                    TileCategoryChange::MoveToSubfolder => (png_in_root, png_in_sub),
                    TileCategoryChange::Delete => (png_in_sub, png_in_root),
                    TileCategoryChange::MoveFromSubfolder => (png_in_sub, png_in_root),
                };
                match std::fs::copy(from, to) {
                    Ok(_) => None,
                    Err(err) => Some((filename, err)),
                }
            });
            for (filename, error) in png_errors {
                errors.push(SerError::IOError {
                    text: format!("could not copy ong for {filename} due to: {error}"),
                    category: category.name.clone(),
                })
            }
        }
    }
    let main_init_path = init.root.join("init.txt");
    if let Err(err) = std::fs::write(main_init_path, main_init_to_write) {
        return Err((
            SerError::IOError {
                text: format!("{err:?}"),
                category: String::from("MAIN"),
            },
            errors,
        ));
    };
    Ok(errors)
}

pub fn backup_init_files(init: &TileInit) -> SerErrorReports {
    let mut res = SerErrorReports::new();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_secs();
    let main_init_path = init.main_init_path();
    let sub_init_paths = init
        .categories
        .iter()
        .filter_map(|cat| cat.filepath().and_then(|fp| Some((cat.clone(), fp))));
    for (category, init_path) in sub_init_paths.chain(
        Some((
            TileCategory {
                enabled: false,
                subfolder: None,
                name: String::from("MAIN_INIT"),
                color: [255, 0, 0],
                tiles: Vec::new(),
                scheduled_change: TileCategoryChange::None,
                index: 0,
                //scheduled_move_to_sub: false,
            },
            main_init_path,
        ))
        .into_iter(),
    ) {
        if init_path.exists() && init_path.is_file() {
            let newpath = init_path
                .parent()
                .expect("could not see parent directory")
                .join(format!("init-backup-{}.txt", timestamp));
            let copy_results = std::fs::copy(init_path, newpath);
            match copy_results {
                Err(err) => {
                    res.push(SerError::IOError {
                        text: format!("{err:?}"),
                        category: category.name,
                    });
                }
                Ok(_) => {}
            }
        }
    }

    res
}

pub fn serialize_category(category: &TileCategory, exclude_disabled: bool) -> Vec<String> {
    let mut res = Vec::new();
    if category.enabled {
        res.push(String::from(CATEGORY_ON_MARKER))
    }
    res.push(serialize_category_header(category));
    for item in category.tiles.iter().filter_map(|tile| {
        if !tile.active && exclude_disabled {
            None
        } else {
            Some(serialize_tileinfo(tile))
        }
    }) {
        res.push(item);
    }
    res
}

pub fn serialize_category_header(category: &TileCategory) -> String {
    let name = category.name.clone();
    let color = aggregate_number_array(category.color.clone().into_iter().map(|num| num as i32));
    let index = category.index;
    format!(r#"-["{name}", color({color})]--CATEGORY_INDEX:{index}"#)
}

pub fn serialize_tileinfo(tile: &TileInfo) -> String {
    let nm = tile.name.clone();
    let sz = aggregate_number_array(tile.size.clone().into_iter());
    let specs = aggregate_specs_array(tile.specs.clone().into_iter());
    let specs2 = tile
        .specs2.clone()
        .and_then(|actual| Some(aggregate_specs_array(actual.into_iter())))
        //.unwrap_or("0")
        ;
    let specs2 = match specs2 {
        Some(actual) => format!("[{}]", actual),
        None => "0".to_string(),
    };
    let tp = tile.tile_type.as_string().unwrap_or("voxelStruct");
    let repeat = tile
        .repeat_layers
        .clone()
        .and_then(|actual| Some(aggregate_number_array(actual.clone().into_iter())))
        .unwrap_or("0".to_string());
    let rnd = tile.random_vars.unwrap_or(1);
    let bf_tiles = tile.buffer_tiles;
    let pt_pos = 0;
    let tags = aggregate_string_array(tile.tags.clone().into_iter());
    let on_tag = match tile.active {
        true => crate::TILE_ON_MARKER,
        false => "",
    };
    format!(
        r#"[#nm:"{nm}", #sz:point({sz}), #specs:[{specs}], #specs2:{specs2}, #tp:"{tp}", #repeatL:[{repeat}], #bfTiles:{bf_tiles}, #rnd:{rnd}, #ptPos:{pt_pos}, #tags:[{tags}]]{on_tag}"#
    )
}

fn aggregate_specs_array<'a>(specs: impl std::iter::Iterator<Item = TileCell>) -> String {
    aggregate_number_array(specs.filter_map(|cell| cell.as_number().ok()))
}

fn aggregate_number_array<'a>(numbers: impl std::iter::Iterator<Item = i32>) -> String {
    aggregate_array(numbers)
}

fn aggregate_string_array<'a>(strings: impl std::iter::Iterator<Item = String>) -> String {
    aggregate_array(strings.map(|str| format!("\"{str}\"")))
}

pub fn aggregate_array(items: impl std::iter::Iterator<Item = impl std::fmt::Display>) -> String {
    items
        .fold(String::new(), |str, new| format!("{str},{new}"))
        .get(1..)
        .unwrap_or("")
        .to_string()
}
