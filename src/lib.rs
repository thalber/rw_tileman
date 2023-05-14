use cycle_map::CycleMap;

mod lingo_de;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UPoint {
    pub x: usize,
    pub y: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    VoxelStruct,
    VoxelStructRockType,
    VoxelStructDisplaceVertical,
    VoxelStructDisplaceHorizontal,
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

impl UPoint {
    pub fn new(x: usize, y: usize) -> UPoint {
        UPoint { x, y }
    }
}

impl TileCell {
    pub fn new(raw_cell: usize) -> Result<TileCell, &'static str> {
        TILE_CELL_NUMBERS.with(|val| match val.get_left(&raw_cell) {
            Some(x) => Ok(x.clone()),
            None => Err("INVALID VALUE"),
        })
    }
    pub fn to_number(&self) -> Result<usize, &'static str> {
        TILE_CELL_NUMBERS.with(|val| match val.get_right(self) {
            Some(x) => Ok(x.clone()),
            None => Err("INVALID VALUE"),
        })
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
}
