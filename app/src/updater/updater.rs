use std::{marker::PhantomData, sync::{Arc, Mutex}, time::Duration};
use anyhow::Result;
use log::*;
use serde::Serialize;
use tauri::{async_runtime::{spawn, JoinHandle}, AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, Updater, UpdaterExt};
use tokio::{sync::TryLockError, time::sleep};

use crate::{shell::ShellManager, updater};

#[derive(Debug, Default, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UpdateStatus {
    #[default]
    Checking,
    Latest,
    Downloading { chunk: usize, length: Option<u64> },
    Downloaded,
    Failed(String),
}

#[cfg(debug_assertions)]
pub type UpdateManager = UpdateManagerImpl<FakeUpdater>;

#[cfg(not(debug_assertions))]
pub type UpdateManager = UpdateManagerImpl<Updater>;

pub struct UpdateManagerImpl<U> {
    app_handle: AppHandle,
    handle: Mutex<Option<JoinHandle<()>>>,
    status: Arc<Mutex<UpdateStatus>>,
    _phantom: PhantomData<U>,
}

impl UpdateManagerImpl<Updater> {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            handle: Mutex::new(None),
            status: Arc::new(Mutex::new(UpdateStatus::Checking)),
            _phantom: PhantomData,
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
            let update_result = updater.check().await;

            match update_result {
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

        let data = update.download(
            move |chunk, length| {
                let mut st = status_chunk.lock().unwrap();
                *st = UpdateStatus::Downloading { chunk, length };
                app_handle.emit("on-update", UpdateStatus::Downloading { chunk, length }).unwrap();
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
        if let Some(handle) = self.handle.lock().unwrap().take() {
            handle.await?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct FakeUpdater;

#[derive(Clone)]
pub struct FakeUpdate {
    pub version: String,
}

impl FakeUpdate {
    pub fn new(app_handle: &AppHandle) -> Self {
        Self { version: app_handle.package_info().version.to_string() }
    }

    pub async fn download<C, D>(&self, mut on_chunk: C, on_finish: D) -> Result<Vec<u8>>
    where
        C: FnMut(usize, Option<u64>) + Send + 'static,
        D: FnOnce() + Send + 'static,
    {
        let total_size = 200 * 1024 * 1024;
        let iterations = 100;
        let chunk_size = total_size / iterations;
        let delay = Duration::from_millis(500);

        for i in 0..iterations {
            on_chunk(i, Some(total_size as u64));
            sleep(delay).await;
        }

        on_finish();

        Ok(vec![0u8; total_size])
    }

    pub fn install(&self, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }
}

impl UpdateManagerImpl<FakeUpdater> {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            handle: Mutex::new(None),
            status: Arc::new(Mutex::new(UpdateStatus::Checking)),
            _phantom: PhantomData,
        }
    }

    pub fn check_updates(&mut self) {
        let status = self.status.clone();
        let app_handle = self.app_handle.clone();

        let handle = spawn(async move {

            let update = FakeUpdate::new(&app_handle);
            {
                let mut st = status.lock().unwrap();
                *st = UpdateStatus::Downloading { chunk: 0, length: Some(1024) };
            }

            let status_chunk = status.clone();
            let status_finish = status.clone();
            let app_handle_chunk = app_handle.clone();
            let app_handle_finish = app_handle.clone();

            let data = update.download(
                move |chunk, length| {
                let mut st = status_chunk.lock().unwrap();
                *st = UpdateStatus::Downloading { chunk, length };
                app_handle_chunk.emit("on-update", UpdateStatus::Downloading { chunk, length }).unwrap();
            },
            move || {
                let mut st = status_finish.lock().unwrap();
                *st = UpdateStatus::Downloaded;
                app_handle_finish.emit("on-update", UpdateStatus::Downloaded).unwrap();
            }).await.unwrap();

            sleep(Duration::from_millis(500)).await;

            update.install(data).unwrap();

            app_handle.emit("on-update", UpdateStatus::Finished).unwrap();
        });

        *self.handle.lock().unwrap() = Some(handle);
    }

    pub fn get_status(&self) -> UpdateStatus {
        self.status.try_lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
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
