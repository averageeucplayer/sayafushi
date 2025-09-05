mod app;
mod live;
mod misc;
mod context;
mod constants;
pub mod parser;
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

use anyhow::Result;
use tauri::Context;

use crate::app::autostart::AutoLaunchManager;
use crate::constants::*;
use crate::context::AppContext;
use crate::database::Database;
use crate::handlers::generate_handlers;
use crate::data::AssetPreloader;
use crate::logger::setup_panic_hook;
use crate::misc::load_windivert;
use crate::settings::SettingsManager;
use crate::ui::on_window_event;
use crate::setup::setup;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    app::init();
    let tauri_context: Context = tauri::generate_context!();
    let package_info = tauri_context.package_info();
    let context = AppContext::new(package_info.version.to_string())?;
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

    setup_panic_hook();

    tauri::Builder::default()
        .manage(loader)
        .manage(context)
        .manage(repository)
        .manage(settings_manager)
        .manage(auto_launch_manager)
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
