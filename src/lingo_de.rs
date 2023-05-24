use lazy_static::lazy_static;

use crate::{app::AppError, *};
use std::collections::HashMap;

//todo: make sure support for negative numbers is not needed

const REGEXSTR_PROPS: &str = r#"\#(\w+):("[\\\w\d\s+_-]*?"|point\([\s\d,-]*?\)|\[\s*((\s*?,?\s*?(-?\d+|"[\w\d\s]*?"))*?)\s*\]|\d+)"#; // selects all flat properties from a tile serialization string. capture group 1 is property name and capture group 2 is property value (then fed to one of the lower regexes)
const REGEXSTR_CATEGORY: &str = r#""(.+?)"\s*?,\s*?color\((.+?)\)"#;
const REGEXSTR_NUMBER: &str = r#"(-?\d+?)"#; //matches unsigned numbers. look at capture group 1 for contents
const REGEXSTR_STRING: &str = r#""([\w\d\s]*?)""#; //matches "-delimited strings. look at capture group 1 for contents
const REGEXSTR_ARRAY: &str = r#"\[(.*?)\]"#; //matches stuff in square brackets. look at capture group 1 for contents
const REGEXSTR_POINT: &str = r#"point\(([\d,]*?)\)"#; //matches lingo points. look at capture group 1  for contents
const REGEXSTR_SPLITCOMMAS: &str = r#"\s*,\s*"#;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum LingoData {
    Number(i32),
    String(String),
    Array(Vec<Box<LingoData>>),
    Point(Vec<i32>),
    InvalidOrNull(String),
}

#[derive(PartialEq, Debug, Clone)]
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
    NoCategory(TileInfo),
    IOError,
    MissingFile,
    MissingValue,
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
        let text = text.trim(); //damn you random whitespaces
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
                    .filter_map(|sub| match sub.trim().parse::<i32>() {
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

pub fn parse_tile_info<'a>(text: &'a str, force_enabled: bool) -> Result<TileInfo, DeserError> {
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
        };
    }
    get_prop!(name, "nm");
    cast_enum!(name, name, "nm", String);
    get_prop!(size, "sz");
    cast_enum!(size, size, "sz", Point);
    get_prop!(specs, "specs");
    get_prop!(specs2, "specs2");
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
        active: force_enabled || text.ends_with(TILE_ON_MARKER),
    };
    Ok(res)
    //Err(DeserError::Todo)
}

pub fn parse_tile_info_multiple<'a>(
    text: &'a str,
) -> Result<(Vec<TileInfo>, DeserErrorReports), DeserError> {
    let mut errors = Vec::new();
    let mut tiles = Vec::new();
    for line in text.lines().filter(|line| !line.starts_with("--")) {
        match parse_tile_info(line, false) {
            Ok(tile) => tiles.push(tile),
            Err(err) => errors.push((line.to_string(), err)),
        }
    }
    return Ok((tiles, errors));
}

pub fn parse_category_header<'a>(text: &'a str) -> Result<TileCategory, DeserError> {
    lazy_static! {
        static ref REGEX_CATEGORY: regex::Regex = regex::Regex::new(REGEXSTR_CATEGORY).unwrap();
        static ref REGEX_SPLITCOMMAS: regex::Regex =
            regex::Regex::new(REGEXSTR_SPLITCOMMAS).unwrap();
    }
    if let Some(caps) = REGEX_CATEGORY.captures(text) {
        let nm = &caps[1];
        let colstr = &caps[2];
        let split = REGEX_SPLITCOMMAS.split(colstr);
        let col: Vec<u8> = split
            .into_iter()
            .filter_map(|sub| sub.parse::<u8>().ok())
            .collect();
        let color = [
            *col.get(0).unwrap_or(&0u8),
            *col.get(1).unwrap_or(&0u8),
            *col.get(2).unwrap_or(&0u8),
        ];

        println!("{:?} {} ({})", color, nm, text);
        Ok(TileCategory::new_main(nm.to_string(), color))
    } else {
        Err(DeserError::Todo)
    }
}

pub fn parse_tile_init<'a>(
    text: String,
    additional_categories: Vec<TileCategory>,
    root: std::path::PathBuf,
) -> Result<TileInit, AppError> {
    let mut errored_lines = Vec::new();
    //let mut success_tiles = Vec::new();
    let mut current_category: TileCategory =
        TileCategory::new_main(String::from("NO_CATEGORY"), [255, 0, 0]);
    //  {
    //     name: "NO_CATEGORY".to_string(),
    //     color: [255, 0, 0],
    //     tiles: Vec::new(),
    //     subfolder: None,
    // };
    let mut categories = Vec::new();
    //let mut results_map = GroupMap::new();

    for line in text.lines().filter(|line| !line.starts_with("--")) {
        // if line.starts_with("--") || line.trim().is_empty() {
        //     continue;
        // } else
        if line.starts_with("-[") && line.ends_with("]") {
            //let maybe_new_category = Err(DeserError::MissingValue);
            let maybe_new_category = parse_category_header(line);
            match maybe_new_category {
                Ok(newcat) => {
                    categories.push(current_category);
                    current_category = newcat;
                }
                Err(err) => errored_lines.push((line.to_string(), err)),
            }
        } else {
            let maybe_new_item = parse_tile_info(line, true);
            match maybe_new_item {
                Ok(new_item) => {
                    current_category.tiles.push(new_item);
                }
                Err(err) => errored_lines.push((line.to_string(), err)),
            }
        }
    }
    categories.push(current_category);
    let additional_categories_clone = additional_categories.clone();
    categories = categories
        .into_iter()
        .filter(|cat| cat.tiles.len() > 0 && !additional_categories_clone.contains(cat))
        .chain(additional_categories)
        .collect();

    Ok(TileInit {
        root,
        categories,
        errored_lines,
    })

    //Err(AppError::Todo)
    //Ok(res)
}

pub fn collect_categories_from_subfolders(
    root: std::path::PathBuf,
) -> Result<Vec<(TileCategory, DeserErrorReports)>, DeserError> {
    lazy_static! {
        static ref REGEX_SPLITCOMMAS: regex::Regex =
            regex::Regex::new(REGEXSTR_SPLITCOMMAS).unwrap();
    }
    let x = std::fs::read_dir(root.clone())
        .into_iter()
        .flatten()
        //.into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let subinit = entry.path().join("init.txt");
            let subcolor = entry.path().join("color.txt");

            if let Ok(contents) = std::fs::read_to_string(subinit.clone()) {
                let color_contents =
                    std::fs::read_to_string(subcolor).unwrap_or(String::from("255,0,0"));
                let mut colorsplit = REGEX_SPLITCOMMAS
                    .split(color_contents.as_str())
                    .filter_map(|substring| substring.parse::<u8>().ok());
                let color = [
                    colorsplit.next().unwrap_or(255u8),
                    colorsplit.next().unwrap_or(0u8),
                    colorsplit.next().unwrap_or(0u8),
                ];
                let enabled = contents.lines().any(|item| item == CATEGORY_ON_MARKER);
                let (tiles, errors) = parse_tile_info_multiple(contents.as_str()).ok()?;
                return Some((
                    TileCategory::new_sub(
                        root.clone(),
                        entry.file_name().to_string_lossy().to_string(),
                        color,
                        tiles,
                        enabled,
                    ),
                    errors,
                ));
            };
            None
            //return std::fs::read_to_string(subinit).ok();
        })
        .collect();
    Ok(x)
}
