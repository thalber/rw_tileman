use eframe::{self, App};
use rw_tileman::app::AppPersistentConfig;

fn main() {
    let wd = std::env::current_dir().expect("could not get wd");
    let default_root = wd.clone();
    let default_out = wd.clone();
    let mut native_options = eframe::NativeOptions::default();
    native_options.multisampling = 0;
    native_options.follow_system_theme = true;
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
                println!("{}", err);
                default_cfg
            }
        },
        Err(err) => {
            println!("{}", err);
            default_cfg
        }
    };
    // .into_iter()
    // .flatten()
    // .next()
    // .unwrap_or(AppPersistentConfig {
    //     root_path: default_root,
    //     output_path: default_out,
    // });

    //let guh = serde_json::de::from_str::<AppPersistentConfig>("s");

    match eframe::run_native(
        "rw_tileman",
        native_options,
        Box::new(|cc| Box::new(rw_tileman::app::TilemanApp::new(cc, cfg).unwrap())),
    ) {
        Ok(_) => {}
        Err(err) => println!("failed to run app: {}", err),
    }
}
