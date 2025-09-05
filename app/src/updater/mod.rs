mod status;
mod manager;
mod fake;

pub use status::UpdateStatus;
pub use manager::UpdateManagerImpl;
pub use fake::{FakeUpdater, FakeUpdateOptions};

#[cfg(debug_assertions)]
pub type UpdateManager = UpdateManagerImpl<FakeUpdater>;

#[cfg(not(debug_assertions))]
pub type UpdateManager = UpdateManagerImpl<tauri_plugin_updater::Updater>;