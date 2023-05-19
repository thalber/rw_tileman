use eframe;

fn main() {
    let root = std::env::current_dir()
        .expect("could not get wd")
        .join("testfiles");
    let out = std::env::current_dir()
        .expect("could not get wd")
        .join("testdumps");
    match eframe::run_native(
        "rw_tileman",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(rw_tileman::app::TilemanApp::new(cc, root, out).unwrap())),
    ) {
        Ok(_) => {}
        Err(err) => println!("failed to run app: {}", err),
    }
}
