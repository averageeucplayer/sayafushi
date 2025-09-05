use std::{marker::PhantomData, sync::{Arc, Mutex}, time::Duration};
use anyhow::Result;
use log::*;
use tauri::{async_runtime::{spawn, JoinHandle}, AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, Updater, UpdaterExt};
use tokio::time::sleep;

use crate::shell::ShellManager;
use super::status::UpdateStatus;

pub struct UpdateManagerImpl<U> {
    pub(crate) app_handle: AppHandle,
    pub(crate) handle: Mutex<Option<JoinHandle<()>>>,
    pub(crate) status: Arc<Mutex<UpdateStatus>>,
    pub(crate) _phantom: PhantomData<U>,

    // This field exists *only* in debug builds (so it won't bloat release).
    // Initialize it in constructors with conditional initializer (shown below).
    #[cfg(debug_assertions)]
    pub(crate) fake_options: Option<crate::updater::fake::FakeUpdateOptions>,
}

impl UpdateManagerImpl<Updater> {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            handle: Mutex::new(None),
            status: Arc::new(Mutex::new(UpdateStatus::Checking)),
            _phantom: PhantomData,
            
            #[cfg(debug_assertions)]
            fake_options: None,
        }
    }

    pub fn check_updates(&mut self) {
        let status = self.status.clone();
        let app_handle = self.app_handle.clone();

        let handle = spawn(async move {
            let updater = match app_handle.updater() {
                Ok(updater) => updater,
                Err(err) => {
                    let mut status = status.lock().unwrap();
                    *status = UpdateStatus::Failed(err.to_string());
                    return;
                },
            };

            match updater.check().await {
                Ok(Some(update)) => {
                    if let Err(err) = Self::on_update(update, &app_handle, status.clone()).await {
                        let mut st = status.lock().unwrap();
                        *st = UpdateStatus::Failed(err.to_string());
                    }
                }
                Ok(None) => {
                    let mut st = status.lock().unwrap();
                    *st = UpdateStatus::Latest;
                }
                Err(err) => {
                    let mut st = status.lock().unwrap();
                    *st = UpdateStatus::Failed(err.to_string());
                }
            }
        });

        *self.handle.lock().unwrap() = Some(handle);
    }

    async fn on_update(update: Update, app_handle: &AppHandle, status: Arc<Mutex<UpdateStatus>>) -> Result<()> {
        let shell_manager = app_handle.state::<ShellManager>();
        info!("update available, downloading: v{}", update.version);

        shell_manager.unload_driver().await;
        shell_manager.remove_driver().await;

        let status_chunk = status.clone();
        let status_finish = status.clone();
        let mut total = 0;

        let data = update.download(
            move |chunk, length| {
                total += chunk;
                let mut st = status_chunk.lock().unwrap();
                let update_status = UpdateStatus::Downloading { chunk: total, length };
                *st = update_status.clone();
                app_handle.emit("on-update", update_status).unwrap();
            },
            move || {
                let mut st = status_finish.lock().unwrap();
                *st = UpdateStatus::Downloaded;
                app_handle.emit("on-update", UpdateStatus::Downloaded).unwrap();
            },
        ).await?;

        sleep(Duration::from_millis(500)).await;
        update.install(data)?;
        Ok(())
    }

    pub fn get_status(&self) -> UpdateStatus {
        self.status.lock().unwrap().clone()
    }

    pub async fn wait(&self) -> Result<()> {
        loop {
            match self.handle.try_lock() {
                Ok(mut guard) => {
                    if let Some(handle) = guard.take() {
                        handle.await?;
                    }
                    break;
                }
                Err(err) => {
                    match err {
                        std::sync::TryLockError::Poisoned(err) => {
                            return Err(anyhow::anyhow!("Lock error: {err}"));
                        },
                        std::sync::TryLockError::WouldBlock => {
                            sleep(Duration::from_secs(1)).await;
                        },
                    }
                }
            }
        }
        Ok(())
    }
}
