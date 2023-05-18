use cycle_map::CycleMap;
use egui::epaint::tessellator::path;
use lingo_de::DeserError;

pub mod app;
pub mod lingo_de;
mod utl;

type ParseErrorReports = Vec<(String, DeserError)>;
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
    Air,
    Wall,
    Slope(usize),
    Floor,
    Entrance,
    Glass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileInit {
    pub categories: Vec<TileCategory>,
    pub errored_lines: ParseErrorReports,
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
        // && self.size == other.size
        // && self.specs == other.specs
        // && self.specs2 == other.specs2
        // && self.tile_type == other.tile_type
        // && self.repeat_layers == other.repeat_layers
        // && self.buffer_tiles == other.buffer_tiles
        // && self.random_vars == other.random_vars
        // && self.preview_pos == other.preview_pos
        // && self.tags == other.tags
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

impl Default for TileInit {
    fn default() -> Self {
        Self {
            categories: Default::default(),
            errored_lines: Default::default(),
        }
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
