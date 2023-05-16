use cycle_map::{CycleMap};
use lingo_de::DeserError;

pub mod app;
pub mod lingo_de;

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
    Air,
    Wall,
    Slope(usize),
    Floor,
    Entrance,
    Glass,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileCategory {
    pub name: String,
    pub color: egui::Color32,
    pub tiles: Vec<TileInfo>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileInit {
    pub categories: Vec<TileCategory>,
    pub errored_lines: Vec<(String, DeserError)>,
}

impl Default for TileInit {
    fn default() -> Self {
        Self {
            categories: Default::default(),
            errored_lines: Default::default(),
        }
    }
}
