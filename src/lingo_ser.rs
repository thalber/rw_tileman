use std::num;

use crate::{TileCell, TileInfo};

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
    format!(
        r#"[#nm:"{nm}", #sz:point({sz}), #specs:[{specs}], #specs2:{specs2}, #tp:"{tp}", #repeatL:[{repeat}], #bfTiles:{bf_tiles}, #rnd:{rnd}, #ptPos:{pt_pos}, #tags:[{tags}]]"#
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
