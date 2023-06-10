use crate::{
    lingo_de::{self, LingoData},
    lingo_ser,
};

#[test]
pub fn ser_and_deser() {
    let initial = r#"[#nm:"test_tile", #sz:point(2, 2), #specs:[1,1,1,1], #specs2:0, #tp:"voxelStruct", #repeatL:[10], #bfTiles:0, #rnd:1, #ptPos:0, #tags:["a", "b"]]"#;
    let de = lingo_de::parse_tile_info(initial, false).unwrap();
    let ser = lingo_ser::serialize_tileinfo(&de);
    //assert_eq!(initial, ser.as_str());
    let de2 = lingo_de::parse_tile_info(ser.as_str(), false).unwrap();
    assert_eq!(de, de2);
}

#[test]
pub fn full_folder_deser() {
    let mut errors = Vec::new();
    let path_in = std::env::current_dir()
        .expect("Could not get working directory")
        .join("testfiles");

    let path_out = std::env::current_dir().unwrap().join("testdumps");
    let additional_categories = lingo_de::collect_categories_from_subfolders(path_in.clone())
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|(category, newerrors)| {
            for newerror in newerrors {
                errors.push(newerror)
            }
            category
        })
        .collect();
    let mut init = lingo_de::parse_tile_init(
        std::fs::read_to_string(path_in.join("init.txt")).unwrap(),
        additional_categories,
        Default::default(),
    )
    .unwrap();
    init.errored_lines = init.errored_lines.into_iter().chain(errors).collect();
    _ = std::fs::write(path_out.join("full_deser_init.txt"), format!("{:#?}", init));
    //_ = std::fs::write(path_in.join("full_deser_errors"), format!("{:#?}", errors));
}

#[test]
pub fn mass_deser_with_categories() {
    //let lingo = std::fs::read_to_string("test_mass_deser.txt").expect("could not read file");
    let des = lingo_de::parse_tile_init(
        std::fs::read_to_string("testfiles/mass_deser.txt").expect("could not read file"),
        Vec::new(),
        Default::default(),
    )
    .unwrap();
    std::fs::write("testdumps/full_deser_out.txt", format!("{:#?}", des))
        .expect("could not write results");
}

#[test]
pub fn mass_deser() {
    let lingo = std::fs::read_to_string("testfiles/mass_deser.txt").expect("could not read file");
    let mut total = 0usize;
    let mut failures = Vec::new();
    //let mut success = 0usize;

    for line in lingo.lines() {
        if line.starts_with("--") || line.starts_with("-[") || line.trim().is_empty() {
            continue;
        }
        total += 1;
        if let Err(err) = lingo_de::parse_tile_info(line, true) {
            failures.push((line, err));
        }
    }
    std::fs::write("testdumps/mass_out.txt", format!("{:#?}", failures))
        .expect("could not write results");
    println!("error on {} out of {}", failures.len(), total);
    //std::fs::write(path, contents)
    //println!("Failed on: {:?}", failures);
    assert_eq!(failures.len(), 0);
}

#[test]
pub fn single_deser() {
    let lingo = std::fs::read_to_string("testfiles/single_deser.txt").expect("could not read file");
    let tileinfo: crate::TileInfo = match lingo_de::parse_tile_info(lingo.as_str(), true) {
        Ok(res) => res,
        Err(e) => {
            let msg = format!(
                "error parsing tileinfo : {:?
            }",
                e
            );
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
