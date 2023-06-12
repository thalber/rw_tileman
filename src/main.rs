#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rw_tileman::*;
fn main() {
    //read config
    let wd = std::env::current_dir().expect("could not get wd");
    let default_root = wd.clone();
    let default_out = wd.clone();
    let mut native_options = eframe::NativeOptions::default();
    native_options.multisampling = 0;
    native_options.follow_system_theme = true;
    native_options.min_window_size = Some(egui::Vec2 { x: 600.0, y: 400.0 });
    let default_cfg = AppPersistentConfig {
        root_path: default_root,
        output_path: default_out,
    };
    let cfg_path = wd.join("tileman_config.json");
    let maybe_cfg = std::fs::read_to_string(cfg_path)
        .map(|text| serde_json::de::from_str::<AppPersistentConfig>(text.as_str()));

    let cfg = match maybe_cfg {
        Ok(maybe_cfg) => match maybe_cfg {
            Ok(actual_cfg) => actual_cfg,
            Err(err) => {
                log::error!("{err}");
                default_cfg
            }
        },
        Err(err) => {
            log::error!("{err}");
            default_cfg
        }
    };
    //initialize logger
    let lhandle = flexi_logger::Logger::try_with_str("debug")
        .unwrap()
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(cfg.output_path.clone())
                .basename("tileman")
                .suffix("log")
                .suppress_timestamp(),
        )
        //.log_to_stdout()
        .write_mode(flexi_logger::WriteMode::BufferAndFlush)
        //.duplicate_to_stdout(flexi_logger::Duplicate::All)
        .use_utc()
        .start()
        .expect("could not create logger");
    log::error!("start");
    //run the app and handle exit error
    match eframe::run_native(
        "rw_tileman",
        native_options,
        Box::new(|cc| {
            let app = rw_tileman::app::TilemanApp::new(cc, cfg, lhandle).unwrap();
            //logger_handle = Some(app._lhandle.clone());
            Box::new(app)
        }),
    ) {
        Ok(_) => {}
        Err(err) => {
            log::error!("failed to run app: {}", err);
        }
    }
}
