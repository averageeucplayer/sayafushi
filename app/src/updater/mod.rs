mod status;
mod manager;

#[cfg(debug_assertions)]
mod fake;

#[cfg(debug_assertions)]
mod internal {
    pub use super::fake::{FakeUpdater, FakeUpdateOptions};
    pub type UpdateManager = super::manager::UpdateManagerImpl<FakeUpdater>;
}

#[cfg(not(debug_assertions))]
mod internal {
    pub use tauri_plugin_updater::Updater;
    pub type UpdateManager = super::manager::UpdateManagerImpl<Updater>;
}

pub use internal::UpdateManager;

#[cfg(debug_assertions)]
pub use internal::FakeUpdateOptions;

pub use status::UpdateStatus;