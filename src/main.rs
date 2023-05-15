use eframe;
use egui;

fn main() {
    match eframe::run_native(
        "Tileman",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(rw_tileman::app::TilemanApp::new(cc))),
    ) {
        Ok(_) => {}
        Err(err) => println!("failed to run app: {}", err),
    }
}
