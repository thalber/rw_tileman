use crate::lingo_de::{self, LingoData};

#[test]
pub fn deser_single_tileinfo() {
    let lingo = std::fs::read_to_string("test_single_deser.txt").expect("could not read file");
    let tileinfo: crate::TileInfo = match lingo_de::parse_tile_info(lingo.as_str()) {
        Ok(res) => res,
        Err(e) => {
            let msg = format!("error parsing tileinfo : {}", e);
            panic!("{msg}")
        }
    };
    println!("{:?}", tileinfo)
}

#[test]
pub fn parse_lingo_values() {
    let testvals = vec![
        r#""Name""#,
        r#"point(2,2)"#,
        "[0,0,0,0]",
        "0",
        r#"["tag1", "tag2"]"#,
    ];
    macro_rules! parse_and_unwrap {
        ($x:expr) => {
            LingoData::parse(testvals[$x]).unwrap()
        };
    }
    let test_string = parse_and_unwrap!(0);
    let test_point = parse_and_unwrap!(1);
    let test_array0 = parse_and_unwrap!(2);
    let test_number = parse_and_unwrap!(3);
    let test_array1 = parse_and_unwrap!(4);
    assert_eq!(test_string, LingoData::String("Name".to_string()));
    assert_eq!(test_point, LingoData::Point(vec![2, 2]));
    assert_eq!(
        test_array0,
        LingoData::Array(vec![
            Box::new(LingoData::Number(0)),
            Box::new(LingoData::Number(0)),
            Box::new(LingoData::Number(0)),
            Box::new(LingoData::Number(0)),
        ])
    );
    assert_eq!(test_number, LingoData::Number(0));
    assert_eq!(
        test_array1,
        LingoData::Array(vec![
            Box::new(LingoData::String("tag1".to_string())),
            Box::new(LingoData::String("tag2".to_string()))
        ])
    )
}
