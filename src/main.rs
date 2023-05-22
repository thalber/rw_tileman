use eframe;

fn main() {
    let root = std::env::current_dir()
        .expect("could not get wd")
        .join("testfiles");
    let out = std::env::current_dir()
        .expect("could not get wd")
        .join("testdumps");
    let mut native_options = eframe::NativeOptions::default();
    native_options.multisampling = 0;
    match eframe::run_native(
        "rw_tileman",
        native_options,
        Box::new(|cc| {
            
            Box::new(rw_tileman::app::TilemanApp::new(cc, root, out).unwrap())}),
    ) {
        Ok(_) => {}
        Err(err) => println!("failed to run app: {}", err),
    }
}
