use crate::lingo_de;

#[test]
pub fn deser_lingo_to_tileinfo() {
    let lingo = std::fs::read_to_string("test_lingo_deser.txt").expect("could not read file");
    let tileinfo: crate::TileInfo = match lingo_de::parse_tile_info(lingo.as_str()) {
        Ok(res) => res,
        Err(e) => {
            let msg = format!("error parsing tileinfo : {}", e);
            panic!("{msg}")
        },
    };
    println!("{:?}", tileinfo)
}