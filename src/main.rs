use eframe;

fn main() {
    match eframe::run_native(
        "rw_tileman",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(rw_tileman::app::TilemanApp::new(cc).unwrap())),
    ) {
        Ok(_) => {}
        Err(err) => println!("failed to run app: {}", err),
    }
}
