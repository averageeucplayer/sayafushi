#[allow(unused_imports)]

mod autostart;
mod abstractions;
mod live;
mod misc;
mod context;
mod constants;
pub mod models;
pub mod database;
mod handlers;
mod setup;
mod logger;
mod settings;
mod shell;
mod background;
mod data;
mod updater;
mod ui;
mod api;
mod local;

use anyhow::Result;
use tauri::Context;

use crate::autostart::AutoLaunchManager;
use crate::constants::*;
use crate::context::AppContext;
use crate::database::Database;
use crate::handlers::generate_handlers;
use crate::data::AssetPreloader;
use crate::local::LocalPlayerRepository;
use crate::logger::{setup_logger, setup_panic_hook};
use crate::misc::load_windivert;
use crate::settings::SettingsManager;
use crate::ui::on_window_event;
use crate::setup::setup;
use log::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    let tauri_context: Context = tauri::generate_context!();
    let package_info = tauri_context.package_info();
    let context = AppContext::new(package_info.version.to_string())?;
    // setup_logger(&context.current_dir);
    setup_panic_hook();
    load_windivert(&context.current_dir).expect("could not load windivert dependencies");
    let auto_launch_manager = AutoLaunchManager::new(
        &package_info.name,
        &context.app_path.display().to_string());
    let loader = AssetPreloader::new();
    let settings_manager = SettingsManager::new(context.settings_path.clone())?;
    let database = Database::new(
        context.database_path.clone(),
        &context.migrations_path,
        &context.version
    ).expect("error setting up database: {}");
    let repository = database.create_repository();
    let local_player = LocalPlayerRepository::new(context.local_player_path.clone())?;

    let log_builder = tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        .level_for("tao::platform_impl::platform::event_loop::runner", LevelFilter::Error)
        .max_file_size(5_000_000)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::LogDir {
                file_name: Some("loa_logs".to_string()),
            },
        ));

    tauri::Builder::default()
        .manage(loader)
        .manage(context)
        .manage(repository)
        .manage(settings_manager)
        .manage(auto_launch_manager)
        .manage(local_player)
        .plugin(log_builder.build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(WINDOW_STATE_FLAGS)
                .build(),
        )
        .setup(setup)
        .on_window_event(on_window_event)
        .invoke_handler(generate_handlers())
        .run(tauri_context)
        .expect("error while running application");

    Ok(())
}
