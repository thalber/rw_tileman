use lazy_static::lazy_static;

use crate::*;
use std::collections::HashMap;

//todo: make sure support for negative numbers is not needed

const REGEXSTR_PROPS: &str =
    r#"\#(\w+):("[\w\d\s]*?"|point\([\d,]*?\)|\[((,?\s?(\d+|"[\w\d\s]*?"))*?)\]|\d+)"#; // selects all flat properties from a tile serialization string. capture group 1 is property name and capture group 2 is property value (then fed to one of the lower regexes)
const REGEXSTR_NUMBER: &str = r#"(\d+?)"#; //matches unsigned numbers. look at capture group 1 for contents
const REGEXSTR_STRING: &str = r#""([\w\d\s]*?)""#; //matches "-delimited strings. look at capture group 1 for contents
const REGEXSTR_ARRAY: &str = r#"\[(.*?)\]"#; //matches stuff in square brackets. look at capture group 1 for contents
const REGEXSTR_POINT: &str = r#"point\(([\d,]*?)\)"#; //matches lingo points. look at capture group 1  for contents
const REGEXSTR_SPLITCOMMAS: &str = r#",\s*"#;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum LingoData {
    Number(usize),
    String(String),
    Array(Vec<Box<LingoData>>),
    Point(Vec<usize>),
    Null,
}

impl LingoData {
    pub fn parse<'a>(text: &str) -> Result<Self, &'a str> {
        // if text == "void" {
        //     return Ok(LingoData::Null);
        // }
        lazy_static! {
            static ref REGEX_NUMBER: regex::Regex = regex::Regex::new(REGEXSTR_NUMBER).unwrap();
            static ref REGEX_STRING: regex::Regex = regex::Regex::new(REGEXSTR_STRING).unwrap();
            static ref REGEX_ARRAY: regex::Regex = regex::Regex::new(REGEXSTR_ARRAY).unwrap();
            static ref REGEX_POINT: regex::Regex = regex::Regex::new(REGEXSTR_POINT).unwrap();
            static ref REGEX_SPLITCOMMAS: regex::Regex =
                regex::Regex::new(REGEXSTR_SPLITCOMMAS).unwrap();
        }
        let mut res = Err("value match not found");
        if let Some(caps) = REGEX_ARRAY.captures(text) {
            let spl = REGEX_SPLITCOMMAS.split(&caps[1]);
            res = Ok(Self::Array(
                spl.into_iter()
                    .filter_map(|sub| match LingoData::parse(sub) {
                        Ok(ld) => Some(Box::new(ld)),
                        Err(_) => None,
                    })
                    .collect(),
            ))
        } else if let Some(caps) = REGEX_POINT.captures(text) {
            let spl = REGEX_SPLITCOMMAS.split(&caps[1]);
            res = Ok(Self::Point(
                spl.into_iter()
                    .filter_map(|sub| match sub.parse::<usize>() {
                        Ok(num) => Some(num),
                        Err(_) => None,
                    })
                    .collect(),
            ))
        } else if let Some(caps) = REGEX_STRING.captures(text) {
            res = Ok(Self::String(String::from(&caps[1])))
        } else if let Some(caps) = REGEX_NUMBER.captures(text) {
            res = match &caps[1].parse::<usize>() {
                Ok(num) => Ok(Self::Number(*num)),
                Err(e) => Err("could not parse number from text"),
            }
        }
        res
    }
    pub fn as_number(&self) -> Option<usize> {
        if let LingoData::Number(num) = self {
            Some(*num)
        } else {
            None
        }
    }
    pub fn as_string(&self) -> Option<String> {
        if let LingoData::String(string) = self {
            Some(string.clone())
        } else {
            None
        }
    }
    pub fn as_string_array(&self) -> Option<Vec<String>> {
        if let LingoData::Array(strings) = self {
            Some(
                strings
                    .iter()
                    .filter_map(|item| {
                        if let Some(str_item) = item.as_string() {
                            Some(str_item)
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
    pub fn as_number_array(&self) -> Option<Vec<usize>> {
        if let LingoData::Array(numbers) = self {
            Some(
                numbers
                    .iter()
                    .filter_map(|item| {
                        if let LingoData::Number(num_item) = **item {
                            Some(num_item)
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
    pub fn as_tilecell_array(&self) -> Option<Vec<TileCell>> {
        let number_array = self.as_number_array();
        if let Some(arr) = number_array {
            return Some(
                arr.into_iter()
                    .map(|item| TileCell::from_number(item))
                    .filter_map(|x| x.ok())
                    .collect(),
            );
        };
        return None;
    }
    pub fn as_null_if_zero(self) -> Self {
        if let LingoData::Number(num_item) = self {
            if num_item == 0 {
                return Self::Null;
            }
        }
        self
    }
}

macro_rules! lookup_static_cyclemap {
    ($map:ident, $func:ident, $lookup:expr) => {
        $map.with(|val| match val.$func($lookup) {
            Some(x) => Ok(x.clone()),
            None => Err("INVALID VALUE"),
        })
    };
}

impl TileCell {
    pub fn from_number(raw_cell: usize) -> Result<TileCell, &'static str> {
        lookup_static_cyclemap!(TILE_CELL_NUMBERS, get_left, &raw_cell)
    }
    pub fn as_number(&self) -> Result<usize, &'static str> {
        lookup_static_cyclemap!(TILE_CELL_NUMBERS, get_right, self)
    }
}

impl TileType {
    pub fn from_string<'a>(text: &'a str) -> Result<TileType, &'static str> {
        lookup_static_cyclemap!(TILE_TYPE_STRINGS, get_left, &text)
    }
    pub fn as_string<'a>(&self) -> Result<&'a str, &'a str> {
        lookup_static_cyclemap!(TILE_TYPE_STRINGS, get_right, self)
    }
}

thread_local! {
    static TILE_CELL_NUMBERS: CycleMap<TileCell, usize> = vec![
        (TileCell::Air, 0),
        (TileCell::Wall, 1),
        (TileCell::Slope(2), 2),
        (TileCell::Slope(3), 3),
        (TileCell::Slope(4), 4),
        (TileCell::Slope(5), 5),
        (TileCell::Floor, 6),
        (TileCell::Entrance, 7),
        (TileCell::Glass, 9)
        ].into_iter().collect();

    static TILE_TYPE_STRINGS: CycleMap<TileType, &'static str> = vec![
        (TileType::VoxelStruct, "voxelStruct"),
        (TileType::VoxelStructRockType, "voxelStructRockType"),
        (TileType::VoxelStructDisplaceV, "voxelStructRandomDisplaceVertical"),
        (TileType::VoxelStructDisplaceH, "voxelStructRandomDisplaceHorizontal"),
        (TileType::Box, "box")

    ].into_iter().collect();
}

pub fn parse_tile_info<'a>(text: &'a str) -> Result<TileInfo, &str> {
    lazy_static::lazy_static! {
        static ref REGEX_PROPERTIES: regex::Regex = regex::Regex::new(REGEXSTR_PROPS).unwrap();
    }
    let mut map: HashMap<String, String> = HashMap::new();
    for cap in REGEX_PROPERTIES.captures_iter(text) {
        let name = &cap[1];
        let val = &cap[2];
        //println!("{name} : {val}")
        map.insert(String::from(name), String::from(val));
    }
    macro_rules! get_prop {
        ($name:ident, $key:literal) => {
            let $name = map
                .get($key)
                .map(|string| string.as_str())
                .unwrap_or("void");
            let $name = LingoData::parse($name);
        };
    }
    macro_rules! cast_enum {
        ($origname:ident, $newname:ident, $key:literal, $entry:ident) => {
            let $newname = if let LingoData::$entry(val) = $origname? {
                Ok(val)
            } else {
                Err(concat!("wrong value type for ", $key))
            };
        };
    }
    let x = map
        .get("a")
        .unwrap_or(&String::from("void"));

    get_prop!(name, "nm");
    cast_enum!(name, name, "nm", String);
    get_prop!(size, "sz");
    cast_enum!(size, size, "sz", Point);
    get_prop!(specs, "specs");
    //cast_enum!(specs, specs, "specs", Array);
    get_prop!(specs2, "specs2");
    //cast_enum!(specs2, specs2_num, "specs2", Number);
    //cast_enum!(specs2, specs2_arr, "specs2", Array);
    get_prop!(tile_type, "tp");
    cast_enum!(tile_type, tile_type, "tp", String);
    get_prop!(repeat_layers, "repeatL"); 
    get_prop!(buffer_tiles, "bfTiles");
    cast_enum!(buffer_tiles, buffer_tiles, "bfTiles", Number);
    get_prop!(random_vars, "rnd");
    cast_enum!(random_vars, random_vars, "rnd", Number);
    get_prop!(preview_pos, "ptPos");
    cast_enum!(preview_pos, preview_pos, "ptPos", Number);
    get_prop!(tags, "tags");
    //cast_enum!(tags, "tags");

    let res = TileInfo {
        name: name?,
        size: size?,
        specs: specs?.as_tilecell_array().ok_or("Specs not an array")?,
        specs2: specs2?.as_null_if_zero().as_tilecell_array(),
        tile_type: TileType::from_string(tile_type?.as_str())?,
        repeat_layers: repeat_layers?.as_number_array(),
        buffer_tiles: buffer_tiles?,
        random_vars: random_vars.ok(),
        preview_pos: preview_pos?,
        tags: tags?.as_string_array().unwrap_or(Vec::new()),
    };
    Ok(res)
    //Err("todo")
}
