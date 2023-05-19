use crate::{SerErrorReports, TileCategory, TileCell, TileInfo, TileInit};

#[derive(Debug, Clone, PartialEq)]
pub enum SerError {
    IOError(String),
    Todo,
}

pub fn rewrite_init(init: &TileInit) -> Result<SerErrorReports, (SerError, SerErrorReports)> {
    println!("{:?}", init.root.to_string_lossy());
    let mut main_init_to_write = String::new();
    let mut errors = SerErrorReports::new();
    for category in init.categories.clone() {
        let cat_text_for_main = serialize_category(&category, true)
            .into_iter()
            .fold(String::new(), |sum, new| format!("{sum}\n{new}"));
        let cat_text_for_sub = serialize_category(&category, false).get(1..).unwrap_or(&[])
            .into_iter()
            .fold(String::new(), |sum, new| format!("{sum}\n{new}"));

        if category.enabled {
            main_init_to_write.push('\n');
            main_init_to_write.push_str(cat_text_for_main.as_str());
        }
        if let Some(sub) = category.subfolder.clone() {
            println!("{}", sub.to_string_lossy());
            let write_result = std::fs::write(sub.join("init.txt"), cat_text_for_sub);
            if let Err(err) = write_result {
                errors.push((category.clone(), SerError::IOError(format!("{:?}", err))));
            }
        };
    }
    if let Err(err) = std::fs::write(init.root.join("init.txt"), main_init_to_write) {
        return Err((SerError::IOError(format!("{err}")), errors));
    };
    Ok(errors)
}

pub fn serialize_category(category: &TileCategory, exclude_disabled: bool) -> Vec<String> {
    // let res = category
    // .tiles
    // .iter()
    // .filter_map(|tile| match tile.active {
    //     true => Some(serialize_tileinfo(tile)),
    //     false => None,
    // }).collect();
    let mut res = Vec::new();
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
    format!(r#"-["{name}", color({color})]"#)
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
    aggregate_array(strings.map(|str| format!("\"{}\"", str)))
}

fn aggregate_array<'a, TI: std::fmt::Display>(
    items: impl std::iter::Iterator<Item = TI>,
) -> String {
    items
        .fold(String::new(), |str, new| format!("{},{}", str, new))
        .get(1..)
        .unwrap_or("")
        .to_string()
}
