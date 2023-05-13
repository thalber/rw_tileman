use std::collections::HashMap;
use crate::*;

//todo: make sure support for negative numbers is not needed

const REGEX_LINGO_PROPS: &str =
    r#"\#(\w+):("[\w\d\s]*?"|point\([\d,]*?\)|\[((,?\s?(\d+|"[\w\d\s]*?"))*?)\]|\d+)"#;
const REGEX_LINGO_NUMBER: &str = r#"(\d+?)"#;
const REGEX_LINGO_STRING: &str = r#""([\w\d\s]*?)""#;
const REGEX_LINGO_ARRAY: &str = r#"\[((,?\s?(\d+|"[\w\d\s]*?"))*?)\]"#;

pub enum LingoPrimitive{
    Number(usize),
    String(String),
    Array(Vec<Box<LingoPrimitive>>),
    Point(Vec<usize>)
}

impl LingoPrimitive {
    pub fn parse<'a>(raw: &String) -> Result<Self, &'a str> {



        Err("todo")
    }
}

pub fn parse_tile_info<'a>(text: &'a str) -> Result<TileInfo, &str> {
    lazy_static::lazy_static! {
        static ref REGEX_PROPERTIES: regex::Regex = regex::Regex::new(REGEX_LINGO_PROPS).unwrap();
    }
    let mut map: HashMap<&'a str, &'a str> = HashMap::new();
    for cap in REGEX_PROPERTIES.captures_iter(text) {
        let name = &cap[1];
        let val = &cap[2];
        println!("{name} : {val}")
        //map.insert(name, val);
    };
    

    // let res = TileInfo {
    //     name: todo!(),
    //     size: todo!(),
    //     specs: todo!(),
    //     specs2: todo!(),
    //     tile_type: todo!(),
    //     repeatL: todo!(),
    //     buffer_tiles: todo!(),
    //     random_vars: todo!(),
    //     ptPos: todo!(),
    //     tags: todo!(),
    // };
    Err("todo")
}
