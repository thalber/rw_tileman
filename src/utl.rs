pub fn indices<'a, T>( vec: &'a Vec<T>) -> impl Iterator<Item = usize> {
    0..vec.len()
}
