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
}

impl LingoData {
    pub fn parse<'a>(text: &str) -> Result<Self, &'a str> {
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
        }
        else if let Some(caps) = REGEX_STRING.captures(text){
            res = Ok(Self::String(String::from(&caps[1])))
        }
        else if let Some(caps) = REGEX_NUMBER.captures(text){
            res = match &caps[1].parse::<usize>(){
                Ok(num) => Ok(Self::Number(*num)),
                Err(e) => Err("could not parse number from text"),
            }
        }
        res
        //Err("todo")
    }
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
    Err("todo")
}
