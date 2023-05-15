use cycle_map::CycleMap;

mod lingo_de;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UPoint {
    pub x: usize,
    pub y: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    VoxelStruct,
    VoxelStructRockType,
    VoxelStructDisplaceV,
    VoxelStructDisplaceH,
    Box
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileInfo {
    pub name: String, //nm
    pub size: Vec<usize>, //sz
    pub specs: Vec<TileCell>, //specs
    pub specs2: Option<Vec<TileCell>>, //specs2
    pub tile_type: TileType, //tp
    pub repeat_layers: Option<Vec<usize>>, //repeatL
    pub buffer_tiles: usize, //bfTiles
    pub random_vars: Option<usize>, //rnd
    pub preview_pos: usize, //ptPos
    pub tags: Vec<String>, //tags
}