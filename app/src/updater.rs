// use std::{pin::Pin, sync::{Arc, Mutex}, thread::sleep, time::Duration};
// use anyhow::{Ok, Result};
// use log::*;
// use serde::Serialize;
// use tauri::{async_runtime::{spawn, JoinHandle}, AppHandle, Emitter, Manager};
// use tauri_plugin_updater::{Update, Updater, UpdaterExt};

// use crate::shell::ShellManager;

// #[derive(Debug, Serialize, Clone)]
// pub enum UpdateStatus {
//     Checking,
//     Latest,
//     Downloading {
//         chunk: usize,
//         length: Option<u64>
//     },
//     Downloaded,
//     Finished,
//     Failed(String)
// }

// pub struct UpdateManager {
//     app_handle: AppHandle,
//     handle: Mutex<Option<JoinHandle<()>>>,
//     status: Arc<Mutex<UpdateStatus>>
// }

// impl UpdateManager {
//     pub fn new(app_handle: AppHandle) -> Self {
//         Self {
//             app_handle,
//             handle: Mutex::new(None),
//             status: Arc::new(Mutex::new(UpdateStatus::Checking))
//         }
//     }

//     pub fn get_status(&self) -> UpdateStatus {
//         self.status.lock().unwrap().clone()
//     }

//     pub fn check_updates(&mut self) {

//         let status = self.status.clone();
//         let app_handle = self.app_handle.clone();

//         let handle = spawn(async move {
//             let updater = match AppUpdaterExtension::updater(&app_handle) {
//                 std::result::Result::Ok(updater) => updater,
//                 Err(err) => {
//                     let mut status = status.lock().unwrap();
//                     *status = UpdateStatus::Failed(err.to_string());
//                     return;
//                 },
//             };

//             let update = match updater.check().await {
//                 std::result::Result::Ok(updater) => updater,
//                 Err(err) => {
//                     let mut status = status.lock().unwrap();
//                     *status = UpdateStatus::Failed(err.to_string());
//                     return;
//                 },
//             };

//             match update {
//                 Some(update) => {
//                     let result = Self::on_update(update, &app_handle, status.clone()).await;

//                     if let Err(err) = result {
//                         let mut status = status.lock().unwrap();
//                         *status = UpdateStatus::Failed(err.to_string());
//                     }
//                 },
//                 None => {
//                     let mut status = status.lock().unwrap();
//                     *status = UpdateStatus::Latest;
//                 },
//             }

//         });

//         *self.handle.get_mut().unwrap() = Some(handle);
//     }

//     async fn on_update(update: impl AppUpdate, app_handle: &AppHandle, status: Arc<Mutex<UpdateStatus>>) -> Result<()> {
//         let shell_manager = app_handle.state::<ShellManager>();

//         info!("update available, downloading update: v{}", update.version());

//         shell_manager.unload_driver().await;
//         shell_manager.remove_driver().await;

//         let status_for_chunk = status.clone();
//         let status_for_finish = status.clone();

//         let data = update.download(
//             move |chunk, length| Self::on_chunk(chunk, length, app_handle, status_for_chunk.clone()),
//             move || Self::on_download_finish(app_handle, status_for_finish)).await?;

//         sleep(Duration::from_millis(500));

//         update.install(data)?;

//         Ok(())
//     }

//     fn on_chunk(chunk: usize, length: Option<u64>, app_handle: &AppHandle, status: Arc<Mutex<UpdateStatus>>) {
//         let mut status = status.lock().unwrap();
//         *status = UpdateStatus::Downloading { chunk, length };
//         app_handle.emit("on-update", UpdateStatus::Downloading { chunk, length }).unwrap();
//     }

//     fn on_download_finish(app_handle: &AppHandle, status: Arc<Mutex<UpdateStatus>>) {
//         let mut status = status.lock().unwrap();
//         *status = UpdateStatus::Downloaded;
//         app_handle.emit("on-update", UpdateStatus::Downloaded).unwrap();
//     }

//     pub async fn wait(&self) -> Result<()> {
//         if let Some(handle) = self.handle.lock().unwrap().take() {
//             handle.await?;
//         }

//         Ok(())
//     }
// }

// // pub struct AppUpdaterWrapper(AppHandle);
// pub struct UpdaterWrapper(AppHandle);

// pub trait AppUpdaterExtension {
//     fn updater(&self) -> Result<impl AppUpdater>;
// }

// pub trait AppUpdate : Send + Sync {
//     fn version(&self) -> String;
//     async fn download<C: FnMut(usize, Option<u64>), D: FnOnce()>(
//         &self,
//         on_chunk: C,
//         on_download_finish: D,
//     ) -> Result<Vec<u8>>;
//     fn install(&self, bytes: impl AsRef<[u8]>) -> Result<()>;
// }


// pub trait AppUpdater : Send + Sync {
//     fn check(&self) -> Pin<Box<dyn Future<Output = Result<Option<Box<dyn AppUpdate + Send>>>> + Send>>;
// }

// impl AppUpdate for Update {
//     async fn download<C: FnMut(usize, Option<u64>), D: FnOnce()>(
//         &self,
//         mut on_chunk: C,
//         on_download_finish: D,
//     ) -> Result<Vec<u8>> {
//         todo!()
//     }

//     fn install(&self, bytes: impl AsRef<[u8]>) -> Result<()> {
//         todo!()
//     }
    
//     fn version(&self) -> String {
//         todo!()
//     }
// }


// impl AppUpdater for Updater {
//     fn check(&self) -> Pin<Box<dyn Future<Output = Result<Option<Box<dyn AppUpdate + Send>>>> + Send>> {
//         Ok(self.check().await?)
//     }
// }

// impl AppUpdaterExtension for AppHandle { 
//     fn updater(&self) -> Result<impl AppUpdater> {
//         Ok(tauri_plugin_updater::UpdaterExt::updater(self)?)
//     }
// }

// // impl AppUpdater for 

// use std::{pin::Pin, sync::{Arc, Mutex}, thread::sleep, time::Duration, future::Future};
// use anyhow::Result;
// use log::*;
// use serde::Serialize;
// use tauri::{async_runtime::{spawn, JoinHandle}, AppHandle, Emitter, Manager};
// use tauri_plugin_updater::{Update, Updater, UpdaterExt};

// use crate::shell::ShellManager;

// #[derive(Debug, Serialize, Clone)]
// pub enum UpdateStatus {
//     Checking,
//     Latest,
//     Downloading {
//         chunk: usize,
//         length: Option<u64>,
//     },
//     Downloaded,
//     Finished,
//     Failed(String),
// }

// pub trait AppUpdate: Send + Sync {
//     fn version(&self) -> String;
//     fn download<'a>(
//         &'a self,
//         on_chunk: impl FnMut(usize, Option<u64>) + Send + 'a,
//         on_download_finish: impl FnOnce() + Send + 'a,
//     ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;

//     fn install(&self, bytes: Vec<u8>) -> Result<()>;
// }

// pub trait AppUpdater: Send + Sync {
//     type Update: AppUpdate + Send + 'static;

//     fn check<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<Option<Self::Update>>> + Send + 'a>>;
// }

// pub trait AppUpdaterExtension {
//     type Updater: AppUpdater;

//     fn updater(&self) -> Result<Self::Updater>;
// }

// pub struct UpdateManager<U> {

// }

// impl UpdateManager<Plugin> {

// }

// impl UpdateManager<Fake> {
    
// }

// pub struct UpdateManager<U: AppUpdate + Send + 'static, Up: AppUpdater<Update = U>> {
//     app_handle: AppHandle,
//     handle: Mutex<Option<JoinHandle<()>>>,
//     status: Arc<Mutex<UpdateStatus>>,
//     _phantom: std::marker::PhantomData<Up>,
// }

// impl<U, Up> UpdateManager<U, Up>
// where
//     U: AppUpdate + Send + 'static,
//     Up: AppUpdater<Update = U> + 'static,
// {
//     pub fn new(app_handle: AppHandle) -> Self {
//         Self {
//             app_handle,
//             handle: Mutex::new(None),
//             status: Arc::new(Mutex::new(UpdateStatus::Checking)),
//             _phantom: std::marker::PhantomData,
//         }
//     }

//     pub fn get_status(&self) -> UpdateStatus {
//         self.status.lock().unwrap().clone()
//     }

//     pub fn check_updates(&mut self, updater: Up) {
//         let status = self.status.clone();
//         let app_handle = self.app_handle.clone();

//         let handle = spawn(async move {
//             let update = match updater.check().await {
//                 Ok(u) => u,
//                 Err(err) => {
//                     let mut status = status.lock().unwrap();
//                     *status = UpdateStatus::Failed(err.to_string());
//                     return;
//                 }
//             };

//             match update {
//                 Some(update) => {
//                     let result = Self::on_update(update, &app_handle, status.clone()).await;

//                     if let Err(err) = result {
//                         let mut status = status.lock().unwrap();
//                         *status = UpdateStatus::Failed(err.to_string());
//                     }
//                 }
//                 None => {
//                     let mut status = status.lock().unwrap();
//                     *status = UpdateStatus::Latest;
//                 }
//             }
//         });

//         *self.handle.lock().unwrap() = Some(handle);
//     }

//     async fn on_update(update: U, app_handle: &AppHandle, status: Arc<Mutex<UpdateStatus>>) -> Result<()> {
//         let shell_manager = app_handle.state::<ShellManager>();

//         info!("update available, downloading update: v{}", update.version());

//         shell_manager.unload_driver().await;
//         shell_manager.remove_driver().await;

//         let status_for_chunk = status.clone();
//         let status_for_finish = status.clone();

//         let data = update
//             .download(
//                 move |chunk, length| Self::on_chunk(chunk, length, app_handle, status_for_chunk.clone()),
//                 move || Self::on_download_finish(app_handle, status_for_finish),
//             )
//             .await?;

//         sleep(Duration::from_millis(500));

//         update.install(data)?;

//         Ok(())
//     }

//     fn on_chunk(chunk: usize, length: Option<u64>, app_handle: &AppHandle, status: Arc<Mutex<UpdateStatus>>) {
//         let mut status = status.lock().unwrap();
//         *status = UpdateStatus::Downloading { chunk, length };
//         app_handle.emit("on-update", UpdateStatus::Downloading { chunk, length }).unwrap();
//     }

//     fn on_download_finish(app_handle: &AppHandle, status: Arc<Mutex<UpdateStatus>>) {
//         let mut status = status.lock().unwrap();
//         *status = UpdateStatus::Downloaded;
//         app_handle.emit("on-update", UpdateStatus::Downloaded).unwrap();
//     }

//     pub async fn wait(&self) -> Result<()> {
//         if let Some(handle) = self.handle.lock().unwrap().take() {
//             handle.await?;
//         }
//         Ok(())
//     }
// }

// impl AppUpdate for Update {
//     fn version(&self) -> String {
//         self.version.clone()
//     }

//     fn download<'a>(
//         &'a self,
//         mut on_chunk: impl FnMut(usize, Option<u64>) + Send + 'a,
//         on_download_finish: impl FnOnce() + Send + 'a,
//     ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
//         Box::pin(async move {
//             // TODO: actual download logic here
//             for i in 0..10 {
//                 on_chunk(i, Some(10));
//             }
//             on_download_finish();
//             Ok(vec![])
//         })
//     }

//     fn install(&self, _bytes: Vec<u8>) -> Result<()> {
//         Ok(())
//     }
// }

// impl AppUpdater for Updater {
//     type Update = Update;

//     fn check<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<Option<Self::Update>>> + Send + 'a>> {
//         Box::pin(async move {
//             let update = self.check().await?;
//             Ok(update)
//         })
//     }
// }

// impl AppUpdaterExtension for AppHandle {
//     type Updater = Updater;

//     fn updater(&self) -> Result<Self::Updater> {
//         Ok(UpdaterExt::updater(self)?)
//     }
// }


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
    Finished,
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
