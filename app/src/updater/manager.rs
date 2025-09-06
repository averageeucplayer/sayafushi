// use std::{sync::Mutex, time::Duration};
// use anyhow::Result;
// use log::*;
// use tauri::{async_runtime::{spawn, JoinHandle}, AppHandle, Manager};
// use tauri_plugin_updater::{Update, Updater, UpdaterExt};
// use tokio::time::sleep;

// use crate::{shell::ShellManager, updater::UpdateStatus};
// use super::status::UpdateStatusHandle;

// pub struct UpdateManagerImpl<U> {
//     pub(crate) app_handle: AppHandle,
//     pub(crate) handle: Mutex<Option<JoinHandle<()>>>,
//     pub(crate) status: UpdateStatusHandle,
//     pub(crate) updater: Option<U>,
// }

// impl UpdateManagerImpl<Updater> {
//     pub fn new(app_handle: AppHandle) -> Self {
//         Self {
//             app_handle: app_handle.clone(),
//             handle: Mutex::new(None),
//             status: UpdateStatusHandle::new(app_handle),
//             updater: None
//         }
//     }

//     pub fn check_updates(&mut self) {
//         let status = self.status.clone();
//         let app_handle = self.app_handle.clone();

//         let handle = spawn(async move {
//             let updater = match app_handle.updater() {
//                 Ok(updater) => updater,
//                 Err(err) => {
//                     error!("An error ocurrest whilst running updater: {}", err);
//                     let update_status = UpdateStatus::Failed(err.to_string());
//                     status.set(update_status);
//                     return;
//                 },
//             };

//             match updater.check().await {
//                 Ok(Some(update)) => {
//                     if let Err(err) = Self::on_update(update, &app_handle, status.clone()).await {
//                         error!("An error ocurrest whilst running updater: {}", err);
//                         let update_status = UpdateStatus::Failed(err.to_string());
//                         status.set(update_status);
//                     }
//                 }
//                 Ok(None) => {
//                     info!("The app is using the latest version");
//                     let update_status = UpdateStatus::Latest;
//                     status.set(update_status);
//                 }
//                 Err(err) => {
//                     error!("An error ocurrest whilst running updater: {}", err);
//                     let update_status = UpdateStatus::Failed(err.to_string());
//                     status.set(update_status);
//                 }
//             }
//         });

//         *self.handle.lock().unwrap() = Some(handle);
//     }

//     async fn on_update(update: Update, app_handle: &AppHandle, status: UpdateStatusHandle) -> Result<()> {
//         let shell_manager = app_handle.state::<ShellManager>();
//         info!("update available, downloading: v{}", update.version);

//         shell_manager.unload_driver().await;
//         shell_manager.remove_driver().await;

//         let status_chunk = status.clone();
//         let status_finish = status.clone();
//         let mut total = 0;

//         let data = update.download(
//             move |chunk, length| {
//                 total += chunk;
//                 let update_status = UpdateStatus::Downloading { chunk: total, length };
//                 status_chunk.set(update_status);
//             },
//             move || {
//                 let update_status = UpdateStatus::Downloaded;
//                 status_finish.set(update_status);
//             },
//         ).await?;

//         sleep(Duration::from_millis(500)).await;
//         update.install(data)?;

//         Ok(())
//     }

//     pub fn get_status(&self) -> UpdateStatus {
//         self.status.get()
//     }

//     pub async fn wait(&self) -> Result<()> {
//         let handle_opt = {
//             let mut guard = self.handle.lock().unwrap();
//             guard.take()
//         };

//         if let Some(handle) = handle_opt {
//             handle.await?;
//         }
        
//         Ok(())
//     }
// }

use std::{marker::PhantomData, sync::Mutex};
use anyhow::Result;
use log::*;
use tauri::{async_runtime::{spawn, JoinHandle}};
use tokio::time::{sleep, Duration};

use super::{UpdateStatus, UpdateStatusHandle, traits::{Updatable, UpdateProvider}};

pub struct UpdateManagerImpl<P, U>
where
    P: UpdateProvider<U>,
    U: Updatable,
{
    pub(crate) handle: Mutex<Option<JoinHandle<()>>>,
    pub(crate) status: UpdateStatusHandle,
    pub(crate) updater: P,
    _marker: PhantomData<U>,
}

impl<P, U> UpdateManagerImpl<P, U>
where
    P: UpdateProvider<U>,
    U: Updatable,
{
    pub fn new(app_handle: tauri::AppHandle, updater: P) -> Self {
        Self {
            handle: Mutex::new(None),
            status: UpdateStatusHandle::new(app_handle),
            updater,
            _marker: PhantomData
        }
    }

    pub fn check_updates(&mut self) {
        let status = self.status.clone();
        let mut updater = match self.updater.setup() {
            Ok(updater) => updater,
            Err(err) => {
                error!("An error ocurrest whilst running updater: {}", err);
                let update_status = UpdateStatus::Failed(err.to_string());
                status.set(update_status);
                return;
            },
        };

        let handle = spawn(async move {
            match updater.check().await {
                Ok(Some(update)) => {
                    if let Err(err) = Self::on_update(update, status.clone()).await {
                        status.set(UpdateStatus::Failed(err.to_string()));
                    }
                }
                Ok(None) => status.set(UpdateStatus::Latest),
                Err(err) => status.set(UpdateStatus::Failed(err.to_string())),
            }
        });

        *self.handle.lock().unwrap() = Some(handle);
    }

    async fn on_update(update: U, status: UpdateStatusHandle) -> Result<()> {
        let status_chunk = status.clone();
        let status_finish = status.clone();
        let mut total = 0;

        let data = update
            .download(
                move |chunk, length| {
                    total += chunk;
                    status_chunk.set(UpdateStatus::Downloading { chunk: total, length });
                },
                move || {
                    status_finish.set(UpdateStatus::Downloaded);
                },
            )
            .await?;

        sleep(Duration::from_millis(500)).await;
        update.install(data)?;
        Ok(())
    }

    pub fn get_status(&self) -> UpdateStatus {
        self.status.get()
    }

    pub async fn wait(&self) -> Result<()> {
        let handle_opt = {
            let mut guard = self.handle.lock().unwrap();
            guard.take()
        };

        if let Some(handle) = handle_opt {
            handle.await?;
        }
        
        Ok(())
    }
}
