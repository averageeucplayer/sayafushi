use std::{error::Error, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use log::*;
use tauri::{async_runtime::spawn, App, AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::{background::{BackgroundWorker, BackgroundWorkerArgs}, constants::DEFAULT_PORT, context::AppContext, extensions::{AppHandleExtensions, WindowExtensions}, settings::{Settings, SettingsManager}, shell::ShellManager, ui::setup_tray};

pub fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {

    let app_handle = app.handle();

    let context = app.state::<AppContext>();
    let shell_manger = ShellManager::new(app_handle.clone(), context.inner().clone());
    let settings_manager = app.state::<SettingsManager>();
    let context = app.state::<AppContext>();
    
    info!("starting app v{}", context.version);
    setup_tray(app_handle)?;

    let update_checked = check_updates(&app_handle);

    let settings = settings_manager.read().unwrap();

    let port = initialize_windows_and_settings(
        &app_handle,
        &settings,
        &shell_manger
    );
    app_handle.manage(shell_manger);

    let mut background = BackgroundWorker::new();

    let args = BackgroundWorkerArgs {
        update_checked,
        app: app_handle.clone(),
        port,
        settings,
        region_file_path: context.region_file_path.clone(),
        version: context.version.clone()
    };

    background.start(args)?;
    app_handle.manage(background);

    // #[cfg(debug_assertions)]
    // {
    //     _logs_window.open_devtools();
    // }

    Ok(())
}

fn check_updates(app_handle: &AppHandle) -> Arc<AtomicBool> {
    let update_checked  = Arc::new(AtomicBool::new(false));
    let checked_clone = update_checked.clone();

    let app_handle = app_handle.clone();
    spawn(async move {
        match app_handle.updater().unwrap().check().await {
            #[cfg(not(debug_assertions))]
            Ok(Some(update)) => {
                info!("update available, downloading update: v{}", update.version);

                unload_driver();
                remove_driver();

                if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                    error!("failed to download update: {}", e);
                }
            }
            Err(e) => {
                warn!("failed to get update: {e}");
                checked_clone.store(true, Ordering::Relaxed);
            }
            _ => {
                info!("no update available");
                checked_clone.store(true, Ordering::Relaxed);
            }
        }
    });

    update_checked
}

fn initialize_windows_and_settings(
    app_handle: &AppHandle,
    settings: &Settings,
    shell_manger: &ShellManager) -> u16 {

    let meter_window = app_handle.get_meter_window().unwrap();
    meter_window.restore_default_state();

    let mini_window = app_handle.get_mini_window().unwrap();
    meter_window.restore_default_state();

    let logs_window = app_handle.get_logs_window().unwrap();
    logs_window.restore_default_state();

    info!("settings loaded");
    if settings.general.mini {
        mini_window.show().unwrap();
    } else if !settings.general.hide_meter_on_start && !settings.general.mini {
        meter_window.show().unwrap();
    }
    if !settings.general.hide_logs_on_start {
        logs_window.show().unwrap();
    }
    if !settings.general.always_on_top {
        meter_window.set_always_on_top(false).unwrap();
        mini_window.set_always_on_top(false).unwrap();
    } else {
        meter_window.set_always_on_top(true).unwrap();
        mini_window.set_always_on_top(true).unwrap();
    }

    let mut port = DEFAULT_PORT;

    if settings.general.auto_iface && settings.general.port > 0 {
        port = settings.general.port;
    }

    if settings.general.start_loa_on_start {
        info!("auto launch game enabled");
        shell_manger.start_loa_process();
    }

    port
}