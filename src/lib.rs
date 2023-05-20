use std::{fmt::Display, path::PathBuf, usize};

use cycle_map::CycleMap;
use lingo_de::DeserError;
use lingo_ser::SerError;

pub mod app;
pub mod lingo_de;
pub mod lingo_ser;
mod utl;

type DeserErrorReports = Vec<(String, DeserError)>;
type SerErrorReports = Vec<(TileCategory, SerError)>;
type PrimitiveColor = [u8; 3];

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    VoxelStruct,
    VoxelStructRockType,
    VoxelStructDisplaceV,
    VoxelStructDisplaceH,
    Box,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileCell {
    Any,
    Air,
    Wall,
    SlopeBottomLeft,
    SlopeBottomRight,
    SlopeTopLeft,
    SlopeTopRight,
    Floor,
    Entrance,
    Glass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileInit {
    pub root: PathBuf,
    pub categories: Vec<TileCategory>,
    pub errored_lines: DeserErrorReports,
}

#[derive(Debug, Clone, Hash)]
pub struct TileCategory {
    pub enabled: bool,
    pub subfolder: Option<std::path::PathBuf>,
    pub name: String,
    pub color: PrimitiveColor,
    pub tiles: Vec<TileInfo>,
}

#[derive(Debug, Clone, Hash)]
pub struct TileInfo {
    pub active: bool,
    pub name: String,                    //nm
    pub size: Vec<i32>,                  //sz
    pub specs: Vec<TileCell>,            //specs
    pub specs2: Option<Vec<TileCell>>,   //specs2
    pub tile_type: TileType,             //tp
    pub repeat_layers: Option<Vec<i32>>, //repeatL
    pub buffer_tiles: i32,               //bfTiles
    pub random_vars: Option<i32>,        //rnd
    pub preview_pos: i32,                //ptPos
    pub tags: Vec<String>,               //tags
}

impl PartialEq for TileInfo {
    fn eq(&self, other: &Self) -> bool {
        //self.active == other.active
        //&&
        self.name == other.name
            && self.size == other.size
            && self.specs == other.specs
            && self.specs2 == other.specs2
            && self.tile_type == other.tile_type
            && self.repeat_layers == other.repeat_layers
            && self.buffer_tiles == other.buffer_tiles
            && self.random_vars == other.random_vars
            //&& self.preview_pos == other.preview_pos
            && self.tags == other.tags
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialEq for TileCategory {
    fn eq(&self, other: &Self) -> bool {
        //self.is_subfolder == other.is_subfolder
        //&&
        self.name == other.name
        //&& self.color == other.color
        //&& self.tiles == other.tiles
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl TileCategory {
    pub fn new_main(name: String, color: PrimitiveColor) -> Self {
        TileCategory {
            enabled: true,
            subfolder: None,
            name,
            color,
            tiles: Vec::new(),
        }
    }
    pub fn new_sub(
        root: std::path::PathBuf,
        name: String,
        color: PrimitiveColor,
        tiles: Vec<TileInfo>,
    ) -> Self {
        TileCategory {
            enabled: true,
            subfolder: Some(root.join(name.clone())),
            name,
            color,
            tiles,
        }
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

    pub fn display_str(&self) -> Result<&'static str, DeserError> {
        lookup_static_cyclemap!(TILE_CELL_DISPLAY, get_right, self)
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

impl TileInfo {
    pub fn display_cells(&self, take_specs2: bool) -> multiarray::Array2D<&'static str> {
        let selected_specs = match take_specs2 {
            false => Some(self.specs.clone()),
            true => self.specs2.clone(),
        };
        if let Some(actual_specs) = selected_specs {
            let xmax = *self.size.get(0).unwrap_or(&1);
            let ymax = *self.size.get(1).unwrap_or(&1);
            let mut res = multiarray::Array2D::new([xmax as usize, ymax as usize], "?_?");
            //let mut specs_iter = actual_specs.into_iter();
            for y in 0..ymax {
                for x in 0..xmax {
                    //let index = 0usize;
                    let index = ((xmax * ymax) - (y + x * ymax + 1)) as usize;
                    let cell = actual_specs.get(index).unwrap_or(&TileCell::Any);
                    let display = cell.display_str().unwrap_or("?^?");
                    res[[x as usize, y as usize]] = display;

                    // 8 5 2
                    // 7 4 1
                    // 6 3 0

                    // (0, 0) (1, 0) (2, 0)
                    // (0, 1) (1, 1) (2, 1)
                    // (0, 2) (1, 2) (2, 2)

                    // 8 - (0x? + 0x1) = 8
                    // 8 - (0x? + 1x1) = 7
                    // 8 - (0x? + 2x1) = 6
                    // 8 - (1x? + 0x1) = 5
                    // 8 - (1x? + 1x1) = 4
                    // 8 - (1x? + 2x1) = 3

                    // let next = specs_iter.next();
                    // let display = next
                    //     .and_then(|cell| Some(cell.display_str().unwrap_or("ERR")))
                    //     .unwrap_or("???");
                    // res.push_str(display);
                }
                //res.push('\n')
            }
            return res;
        };
        multiarray::Array2D::new([0, 0], "")
    }
}

const TILE_ON_MARKER: &str = "--TILE_ENABLED";

thread_local! {
    static TILE_CELL_NUMBERS: CycleMap<TileCell, i32> = vec![
        (TileCell::Any, -1),
        (TileCell::Air, 0),
        (TileCell::Wall, 1),
        (TileCell::SlopeBottomLeft, 2),
        (TileCell::SlopeBottomRight, 3),
        (TileCell::SlopeTopLeft, 4),
        (TileCell::SlopeTopRight, 5),
        (TileCell::Floor, 6),
        (TileCell::Entrance, 7),
        (TileCell::Glass, 9)
        ].into_iter().collect();

    static TILE_CELL_DISPLAY: CycleMap<TileCell, &'static str> = vec![
        (TileCell::Any, "..."),
        (TileCell::Air, "   "),
        (TileCell::Wall, "[ ]"),
        (TileCell::SlopeBottomLeft, " \\|"),
        (TileCell::SlopeBottomRight, "|/ "),
        (TileCell::SlopeTopLeft, " /|"),
        (TileCell::SlopeTopRight, "|\\ "),
        (TileCell::Floor, "==="),
        (TileCell::Entrance, "< >"),
        (TileCell::Glass, "{ }")
        ].into_iter().collect();

    static TILE_TYPE_STRINGS: CycleMap<TileType, &'static str> = vec![
        (TileType::VoxelStruct, "voxelStruct"),
        (TileType::VoxelStructRockType, "voxelStructRockType"),
        (TileType::VoxelStructDisplaceV, "voxelStructRandomDisplaceVertical"),
        (TileType::VoxelStructDisplaceH, "voxelStructRandomDisplaceHorizontal"),
        (TileType::Box, "box")

    ].into_iter().collect();
}
