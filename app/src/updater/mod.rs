#![allow(dead_code)]

pub mod status;
pub mod manager;
pub mod traits;
pub mod real;

#[cfg(feature = "develop")]
pub mod fake;

pub use status::{UpdateStatus, UpdateStatusHandle};
pub use manager::UpdateManagerImpl;
use tauri::{Manager, AppHandle};

use crate::updater::{fake::{FakeUpdate, FakeUpdater}, real::{AppUpdate, AppUpdater}};

#[cfg(feature = "develop")]
pub type FakeUpdateManager = UpdateManagerImpl<FakeUpdater, FakeUpdate>;

#[cfg(feature = "develop")]
use crate::updater::fake::*;

pub type AppUpdateManager = UpdateManagerImpl<AppUpdater, AppUpdate>;

#[cfg(not(feature = "develop"))]
pub type UpdateManager = AppUpdateManager;

#[cfg(feature = "develop")]
pub type UpdateManager = FakeUpdateManager;

pub fn setup_updater(app_handle: &AppHandle) {
    #[cfg(feature = "develop")]
    {

        let updater = FakeUpdater {
            app_handle: app_handle.clone(),
            options: FakeUpdateOptions::Latest,
        };

        let mut update_manager = UpdateManager::new(app_handle.clone(), updater);
        update_manager.check_updates();
        app_handle.manage(update_manager);
    }

    #[cfg(all(feature = "production"))]
    {
        use crate::updater::AppUpdater;

        let updater = AppUpdater::new(app_handle);
        let mut update_manager = UpdateManager::new(app_handle.clone(), updater);
        update_manager.check_updates();
        app_handle.manage(update_manager);
    }
}