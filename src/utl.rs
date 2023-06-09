pub fn indices<'a, T>( vec: &'a Vec<T>) -> impl Iterator<Item = usize> {
    0..vec.len()
}

pub fn name_matches_search(item: &String, search_selection: &String) -> bool {
    item.to_lowercase()
        .contains(search_selection.as_str().to_lowercase().as_str())
}