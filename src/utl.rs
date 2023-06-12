use serde::__private::de;

use crate::{app::AppError, PrimitiveColor, TileCell};

pub fn indices<'a, T>(vec: &'a Vec<T>) -> impl Iterator<Item = usize> {
    0..vec.len()
}

pub fn name_matches_search(item: &String, search_selection: &String) -> bool {
    item.to_lowercase()
        .contains(search_selection.as_str().to_lowercase().as_str())
}

pub fn read_cell_texture(cell: TileCell) -> Result<multiarray::Array2D<PrimitiveColor>, AppError> {
    let path = format!("{cell:?}.png");
    let mut res = multiarray::Array2D::new(
        [crate::CELL_TEXTURE_DIM, crate::CELL_TEXTURE_DIM],
        [000, 000, 000],
    );
    let file = crate::ASSETS_DIR
        .get_file(path.clone())
        .ok_or(AppError::MissingTexture(path.clone()))?;
    let mut reader = png::Decoder::new(file.contents())
        .read_info()
        .map_err(|err| AppError::InvalidTexture(path.clone(), err))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    //let mut buf = Vec::new();
    reader
        .next_frame(&mut buf)
        .map_err(|err| AppError::InvalidTexture(path.clone(), err))?;
    let mut buf = buf.into_iter();
    for y in 0..crate::CELL_TEXTURE_DIM {
        for x in 0..crate::CELL_TEXTURE_DIM {
            let r = buf
                .next()
                .ok_or_else(|| AppError::TextureNotLargeEnough(path.clone()))?;
            let g = buf
                .next()
                .ok_or_else(|| AppError::TextureNotLargeEnough(path.clone()))?;
            let b = buf
                .next()
                .ok_or_else(|| AppError::TextureNotLargeEnough(path.clone()))?;
            res[[x, y]] = [r, g, b]
        }
    }
    Ok(res)
}
