#![allow(dead_code)]
#![allow(unused_imports)]

pub mod status;
pub mod manager;
pub mod traits;
pub mod real;
pub mod fake;

pub use status::{UpdateStatus, UpdateStatusHandle};
pub use manager::UpdateManagerImpl;
use tauri::{Manager, AppHandle};

#[cfg(all(not(feature = "develop"), not(feature = "production")))]
use crate::updater::fake::{FakeUpdateOptions, FakeUpdate, FakeUpdater};

#[cfg(all(not(feature = "develop"), not(feature = "production")))]
pub type UpdateManager = UpdateManagerImpl<FakeUpdater, FakeUpdate>;

#[cfg(feature = "develop")]
use crate::updater::fake::{FakeUpdate, FakeUpdater};

#[cfg(feature = "develop")]
pub type FakeUpdateManager = UpdateManagerImpl<FakeUpdater, FakeUpdate>;

#[cfg(feature = "develop")]
pub type UpdateManager = FakeUpdateManager;

#[cfg(feature = "production")]
use crate::updater::real::{AppUpdate, AppUpdater};

#[cfg(feature = "production")]
pub type AppUpdateManager = UpdateManagerImpl<AppUpdater, AppUpdate>;

#[cfg(feature = "production")]
pub type UpdateManager = AppUpdateManager;

pub fn setup_updater(app_handle: &AppHandle) {
    #[cfg(any(feature = "develop", not(feature = "production")))]
    {
        use crate::updater::fake::FakeUpdateOptions;
        
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

        let updater = AppUpdater::new(app_handle.clone());
        let mut update_manager = UpdateManager::new(app_handle.clone(), updater);
        update_manager.check_updates();
        app_handle.manage(update_manager);
    }
}