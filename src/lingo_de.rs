use lazy_static::lazy_static;

use crate::*;
use std::collections::HashMap;

//todo: make sure support for negative numbers is not needed

const REGEXSTR_PROPS: &str =
    r#"\#(\w+):("[\\\w\d\s+_-]*?"|point\([\s\d,-]*?\)|\[\s*((\s*?,?\s*?(-?\d+|"[\w\d\s]*?"))*?)\s*\]|\d+)"#; // selects all flat properties from a tile serialization string. capture group 1 is property name and capture group 2 is property value (then fed to one of the lower regexes)
const REGEXSTR_NUMBER: &str = r#"(-?\d+?)"#; //matches unsigned numbers. look at capture group 1 for contents
const REGEXSTR_STRING: &str = r#""([\w\d\s]*?)""#; //matches "-delimited strings. look at capture group 1 for contents
const REGEXSTR_ARRAY: &str = r#"\[(.*?)\]"#; //matches stuff in square brackets. look at capture group 1 for contents
const REGEXSTR_POINT: &str = r#"point\(([\d,]*?)\)"#; //matches lingo points. look at capture group 1  for contents
const REGEXSTR_SPLITCOMMAS: &str = r#",\s*"#;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum LingoData {
    Number(i32),
    String(String),
    Array(Vec<Box<LingoData>>),
    Point(Vec<i32>),
    InvalidOrNull(String),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DeserError {
    RegexMatchFailed(String),
    ContentsNotParsed(String),
    DataConvertFailed(String),
    TypeMismatch {
        key: String,
        expected: String,
        got: String,
    },
    InvalidValue(String),
    Todo,
}

impl LingoData {
    pub fn parse<'a>(text: &str) -> Result<Self, DeserError> {
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
        let mut res = Ok(LingoData::InvalidOrNull(text.to_string()));
        if text.starts_with("[") && text.ends_with("]") {
            let spl = REGEX_SPLITCOMMAS.split(&text[1..text.len() - 1]);
            res = Ok(Self::Array(
                spl.into_iter()
                    .filter_map(|sub| match LingoData::parse(sub) {
                        Ok(ld) => Some(Box::new(ld)),
                        Err(_) => None,
                    })
                    .collect(),
            ))
        } else if text.starts_with("point(") && text.ends_with(")") {
            let spl = REGEX_SPLITCOMMAS.split(&text[6..text.len() - 1]);
            res = Ok(Self::Point(
                spl.into_iter()
                    .filter_map(|sub| match sub.parse::<i32>() {
                        Ok(num) => Some(num),
                        Err(_) => None,
                    })
                    .collect(),
            ))
        } else if text.starts_with("\"") && text.ends_with("\"") {
            res = Ok(LingoData::String(String::from(&text[1..text.len() - 1])))
        } else if let Ok(val) = text.parse::<i32>() {
            res = Ok(LingoData::Number(val))
        }

        //Err(DeserError::RegexMatchFailed(text.to_string()));
        // if let Some(caps) = REGEX_ARRAY.captures(text) {
        //     let spl = REGEX_SPLITCOMMAS.split(&caps[1]);
        //     res = Ok(Self::Array(
        //         spl.into_iter()
        //             .filter_map(|sub| match LingoData::parse(sub) {
        //                 Ok(ld) => Some(Box::new(ld)),
        //                 Err(_) => None,
        //             })
        //             .collect(),
        //     ))
        // } else if let Some(caps) = REGEX_POINT.captures(text) {
        //     let spl = REGEX_SPLITCOMMAS.split(&caps[1]);
        //     res = Ok(Self::Point(
        //         spl.into_iter()
        //             .filter_map(|sub| match sub.parse::<i32>() {
        //                 Ok(num) => Some(num),
        //                 Err(_) => None,
        //             })
        //             .collect(),
        //     ))
        // } else if let Some(caps) = REGEX_STRING.captures(text) {
        //     res = Ok(Self::String(String::from(&caps[1])))
        // } else if let Some(caps) = REGEX_NUMBER.captures(text) {
        //     res = match &caps[1].parse::<i32>() {
        //         Ok(num) => Ok(Self::Number(*num)),
        //         Err(e) => Err(DeserError::ContentsNotParsed(format!(
        //             "{} (usize)",
        //             &caps[1]
        //         ))),
        //     }
        // }
        res
    }
    pub fn as_number(&self) -> Result<i32, DeserError> {
        if let LingoData::Number(num) = self {
            Ok(*num)
        } else {
            Err(DeserError::DataConvertFailed(format!(
                "{:?} not a number",
                self
            )))
        }
    }
    pub fn as_string(&self) -> Result<String, DeserError> {
        if let LingoData::String(string) = self {
            Ok(string.clone())
        } else {
            Err(DeserError::DataConvertFailed(format!(
                "{:?} not a string",
                self
            )))
        }
    }
    pub fn as_string_array(&self) -> Result<Vec<String>, DeserError> {
        if let LingoData::Array(strings) = self {
            Ok(strings
                .iter()
                .filter_map(|item| {
                    if let Ok(str_item) = item.as_string() {
                        Some(str_item)
                    } else {
                        None
                    }
                })
                .collect())
        } else {
            Err(DeserError::DataConvertFailed(format!(
                "could not build StringArray from {:?}",
                self
            )))
        }
    }
    pub fn as_number_array(&self) -> Result<Vec<i32>, DeserError> {
        if let LingoData::Array(numbers) = self {
            Ok(numbers
                .iter()
                .filter_map(|item| {
                    if let LingoData::Number(num_item) = **item {
                        Some(num_item)
                    } else {
                        None
                    }
                })
                .collect())
        } else {
            Err(DeserError::DataConvertFailed(format!(
                "could not build NumberArray from {:?}",
                self
            )))
        }
    }
    pub fn as_tilecell_array(&self) -> Result<Vec<TileCell>, DeserError> {
        let number_array = self.as_number_array();
        if let Ok(arr) = number_array {
            return Ok(arr
                .into_iter()
                .map(|item| TileCell::from_number(item))
                .filter_map(|x| x.ok())
                .collect());
        };
        Err(DeserError::DataConvertFailed(format!(
            "could not build tilecellArray from {:?}",
            self
        )))
    }
    pub fn as_null_if_zero(self) -> Self {
        if let LingoData::Number(num_item) = self {
            if num_item == 0 {
                return Self::InvalidOrNull("NULL".to_string());
            }
        }
        self
    }
}

macro_rules! lookup_static_cyclemap {
    ($map:ident, $func:ident, $lookup:expr) => {
        $map.with(|val| match val.$func($lookup) {
            Some(x) => Ok(x.clone()),
            None => Err(DeserError::InvalidValue(format!("invalid value {:?}", val))),
        })
    };
}

impl TileCell {
    pub fn from_number(raw_cell: i32) -> Result<TileCell, DeserError> {
        lookup_static_cyclemap!(TILE_CELL_NUMBERS, get_left, &raw_cell)
    }
    pub fn as_number(&self) -> Result<i32, DeserError> {
        lookup_static_cyclemap!(TILE_CELL_NUMBERS, get_right, self)
    }
}

impl TileType {
    pub fn from_string<'a>(text: &'a str) -> Result<TileType, DeserError> {
        lookup_static_cyclemap!(TILE_TYPE_STRINGS, get_left, &text)
    }
    pub fn as_string<'a>(&self) -> Result<&'a str, DeserError> {
        lookup_static_cyclemap!(TILE_TYPE_STRINGS, get_right, self)
    }
}

thread_local! {
    static TILE_CELL_NUMBERS: CycleMap<TileCell, i32> = vec![
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

pub fn parse_tile_info<'a>(text: &'a str) -> Result<TileInfo, DeserError> {
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
                .unwrap_or(concat!("WARNING: MISSING ITEM ", $key));
            let $name = LingoData::parse($name);
        };
    }
    macro_rules! cast_enum {
        ($origname:ident, $newname:ident, $key:literal, $entry:ident) => {
            let $newname = match $origname {
                Ok(LingoData::$entry(val)) => Ok(val),
                Ok(val) => Err(DeserError::TypeMismatch {
                    key: $key.to_string(),
                    expected: stringify!($entry).to_string(),
                    got: format!("{:?}", val),
                }),
                Err(err) => Err(err),
            };
            // if let Ok(LingoData::$entry(val)) = $origname {
            //     Ok(val)
            // } else {
            //     Err(DeserError::TypeMismatch{
            //         key: $key.to_string(),
            //         expected: "$entry".to_string(),
            //         got:
            //     })
            // };
        };
    }
    //let x = map.get("a").unwrap_or(&String::from("void"));

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
        specs: specs?.as_tilecell_array()?,
        specs2: specs2?.as_null_if_zero().as_tilecell_array().ok(),
        tile_type: TileType::from_string(tile_type?.as_str())?,
        repeat_layers: repeat_layers.and_then(|x| x.as_number_array()).ok(),
        buffer_tiles: buffer_tiles?,
        random_vars: random_vars.ok(),
        preview_pos: preview_pos?,
        tags: tags?.as_string_array().unwrap_or(Vec::new()),
    };
    // let res = TileInfo {
    //     name: name?,
    //     size: size?,
    //     specs: specs?.as_tilecell_array().ok_or("Specs not an array")?,
    //     specs2: specs2?.as_null_if_zero().as_tilecell_array(),
    //     tile_type: TileType::from_string(tile_type?.as_str())?,
    //     repeat_layers: repeat_layers?.as_number_array(),
    //     buffer_tiles: buffer_tiles?,
    //     random_vars: random_vars.ok(),
    //     preview_pos: preview_pos?,
    //     tags: tags?.as_string_array().unwrap_or(Vec::new()),
    // };
    // Ok(res)

    Ok(res)
    //Err(DeserError::Todo)
}
