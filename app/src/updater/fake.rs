// #![allow(dead_code)]

// use std::{
//     fs::File,
//     io::{BufReader, Read},
//     path::PathBuf,
//     sync::Mutex,
//     time::Duration,
// };
// use anyhow::{anyhow, Result};
// use tauri::{async_runtime::spawn, AppHandle};
// use tauri_plugin_opener::OpenerExt;
// use tokio::time::sleep;
// use log::*;

// use super::status::{UpdateStatus, UpdateStatusHandle};
// use super::manager::UpdateManagerImpl;

// /// Configurable options for the fake updater.
// #[derive(Clone)]
// pub enum FakeUpdateOptions {
//     Failed,
//     Latest,
//     /// Simulate a download with synthetic data.
//     Synthetic {
//         with_total_header: bool,
//         total_size: u64,
//         iterations: usize,
//         delay: Duration,
//     },
//     /// Stream from an actual binary file on disk.
//     Binary {
//         path: PathBuf,
//         with_total_header: bool,
//         delay: Duration,
//     },
// }

// impl Default for FakeUpdateOptions {
//     fn default() -> Self {
//         FakeUpdateOptions::Synthetic {
//             with_total_header: true,
//             total_size: 200 * 1024 * 1024, // 200 MB
//             iterations: 25,
//             delay: Duration::from_millis(100),
//         }
//     }
// }

// #[derive(Clone)]
// pub struct FakeUpdater {
//     app_handle: AppHandle,
//     options: FakeUpdateOptions,
// }

// impl FakeUpdater {
//     pub async fn check(&self) -> Result<Option<FakeUpdate>> {
//         match &self.options {
//             FakeUpdateOptions::Failed => Err(anyhow!("Fake update error")),
//             FakeUpdateOptions::Latest => Ok(None),
//             _ => Ok(Some(FakeUpdate::new(self.app_handle.clone(), self.options.clone()))),
//         }
//     }
// }

// #[derive(Clone)]
// pub struct FakeUpdate {
//     pub app_handle: AppHandle,
//     pub options: FakeUpdateOptions,
// }

// impl FakeUpdate {
//     pub fn new(app_handle: AppHandle, options: FakeUpdateOptions) -> Self {
//         Self { app_handle, options }
//     }

//     pub async fn download<C, D>(&self, mut on_chunk: C, on_finish: D) -> Result<Vec<u8>>
//     where
//         C: FnMut(usize, Option<u64>) + Send + 'static,
//         D: FnOnce() + Send + 'static,
//     {
//         match &self.options {
//             FakeUpdateOptions::Binary { path, with_total_header, delay } => {
//                 let mut file = BufReader::new(File::open(path)?);
//                 let mut buf = vec![0u8; 8192];
//                 let mut data = Vec::new();
//                 let mut chunk_size = 0;
//                 let total_size = file.get_ref().metadata()?.len();

//                 loop {
//                     let n = file.read(&mut buf)?;
//                     if n == 0 {
//                         break;
//                     }

//                     chunk_size += n;
//                     data.extend_from_slice(&buf[..n]);
//                     on_chunk(chunk_size, with_total_header.then_some(total_size));
//                     sleep(*delay).await;
//                 }

//                 on_finish();
//                 Ok(data)
//             }
//             FakeUpdateOptions::Synthetic { with_total_header, total_size, iterations, delay } => {
//                 let total_size_arg = with_total_header.then_some(*total_size);
//                 let chunk_size = (total_size / *iterations as u64) as usize;
//                 let last_chunk_size = (total_size % *iterations as u64) as usize;

//                 let mut data = Vec::with_capacity(*total_size as usize);

//                 for _ in 0..*iterations {
//                     data.extend(vec![0u8; chunk_size]);
//                     on_chunk(chunk_size, total_size_arg);
//                     sleep(*delay).await;
//                 }

//                 if last_chunk_size > 0 {
//                     data.extend(vec![0u8; last_chunk_size]);
//                     on_chunk(last_chunk_size, total_size_arg);
//                 }

//                 on_finish();
//                 Ok(data)
//             }
//             _ => unreachable!(),
//         }
//     }

//     pub fn install(&self, _data: Vec<u8>) -> Result<()> {
//         if let FakeUpdateOptions::Binary { path, .. } = &self.options {
//             self.app_handle.opener().open_path(path.to_string_lossy(), None::<String>)?;
//         }
//         Ok(())
//     }
// }

// impl UpdateManagerImpl<FakeUpdater> {
//     pub fn new(app_handle: AppHandle, options: FakeUpdateOptions) -> Self {
//         Self {
//             app_handle: app_handle.clone(),
//             handle: Mutex::new(None),
//             status: UpdateStatusHandle::new(app_handle.clone()),
//             updater: Some(FakeUpdater { app_handle, options }),
//         }
//     }

//     pub fn check_updates(&mut self) {
//         let status = self.status.clone();
//         let updater = self.updater.clone().unwrap();

//         let handle = spawn(async move {
//             match updater.check().await {
//                 Ok(Some(update)) => {
//                     if let Err(err) = Self::on_update(update, status.clone()).await {
//                         error!("Fake updater error: {err}");
//                         status.set(UpdateStatus::Failed(err.to_string()));
//                     }
//                 }
//                 Ok(None) => {
//                     info!("The app is using the latest version");
//                     status.set(UpdateStatus::Latest);
//                 }
//                 Err(err) => {
//                     error!("Fake updater error: {err}");
//                     status.set(UpdateStatus::Failed(err.to_string()));
//                 }
//             }
//         });

//         *self.handle.lock().unwrap() = Some(handle);
//     }

//     async fn on_update(update: FakeUpdate, status: UpdateStatusHandle) -> Result<()> {
//         info!("fake update available, simulating download...");
//         let status_chunk = status.clone();
//         let status_finish = status.clone();
//         let mut total = 0;

//         let data = update
//             .download(
//                 move |chunk, length| {
//                     total += chunk;
//                     status_chunk.set(UpdateStatus::Downloading { chunk: total, length });
//                 },
//                 move || {
//                     status_finish.set(UpdateStatus::Downloaded);
//                 },
//             )
//             .await?;

//         sleep(Duration::from_millis(500)).await;

//         update.install(data)?;
//         Ok(())
//     }

//     pub fn get_status(&self) -> UpdateStatus {
//         self.status.get()
//     }
// }

use std::{fs::File, io::{BufReader, Read}, path::PathBuf, time::Duration};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tokio::time::sleep;

use super::traits::{Updatable, UpdateProvider};

#[derive(Clone)]
pub enum FakeUpdateOptions {
    FailedInvalidConfig,
    Failed,
    Latest,
    Synthetic {
        with_total_header: bool,
        total_size: u64,
        iterations: usize,
        delay: Duration,
    },
    Binary {
        path: PathBuf,
        with_total_header: bool,
        delay: Duration,
    },
}

impl Default for FakeUpdateOptions {
    fn default() -> Self {
        Self::Synthetic {
            with_total_header: true,
            total_size: 200 * 1024 * 1024,
            iterations: 25,
            delay: Duration::from_millis(100),
        }
    }
}

#[derive(Clone)]
pub struct FakeUpdater {
    pub app_handle: AppHandle,
    pub options: FakeUpdateOptions,
}

#[derive(Clone)]
pub struct FakeUpdate {
    pub app_handle: AppHandle,
    pub options: FakeUpdateOptions,
}

#[async_trait]
impl Updatable for FakeUpdate {
    async fn download<C, D>(&self, mut on_chunk: C, on_finish: D) -> Result<Vec<u8>>
    where
        C: FnMut(usize, Option<u64>) + Send + 'static,
        D: FnOnce() + Send + 'static,
    {
        match &self.options {
            FakeUpdateOptions::Binary { path, with_total_header, delay } => {
                let mut file = BufReader::new(File::open(path)?);
                let mut buf = vec![0u8; 8192];
                let mut data = Vec::new();
                let mut chunk_idx = 0;
                let total_size = file.get_ref().metadata()?.len();

                loop {
                    let n = file.read(&mut buf)?;
                    if n == 0 { break; }

                    data.extend_from_slice(&buf[..n]);
                    on_chunk(chunk_idx, with_total_header.then_some(total_size));
                    chunk_idx += 1;
                    sleep(*delay).await;
                }

                on_finish();
                Ok(data)
            }
            FakeUpdateOptions::Synthetic { with_total_header, total_size, iterations, delay } => {
                let total_size_arg = with_total_header.then_some(*total_size);
                let chunk_size = (total_size / *iterations as u64) as usize;
                let last_chunk_size = (total_size % *iterations as u64) as usize;

                let mut data = Vec::with_capacity(*total_size as usize);

                for _ in 0..*iterations {
                    data.extend(vec![0u8; chunk_size]);
                    on_chunk(chunk_size, total_size_arg);
                    sleep(*delay).await;
                }

                if last_chunk_size > 0 {
                    data.extend(vec![0u8; last_chunk_size]);
                    on_chunk(last_chunk_size, total_size_arg);
                }

                on_finish();
                Ok(data)
            }
            _ => unreachable!(),
        }
    }

    fn install(&self, _data: Vec<u8>) -> Result<()> {
        if let FakeUpdateOptions::Binary { path, .. } = &self.options {
            self.app_handle.opener().open_path(path.to_string_lossy(), None::<String>)?;
        }
        Ok(())
    }
}

#[async_trait]
impl UpdateProvider<FakeUpdate> for FakeUpdater {
    fn setup(&self) -> Result<Self> {
        match &self.options {
            FakeUpdateOptions::FailedInvalidConfig => Err(anyhow!("Fake update error - invalid config")),
            options => Ok(Self { options: options.clone(), app_handle: self.app_handle.clone() })
        }
    }

    async fn check(&mut self) -> Result<Option<FakeUpdate>> {
        match &self.options {
            FakeUpdateOptions::Failed => Err(anyhow!("Fake update error")),
            FakeUpdateOptions::Latest => Ok(None),
            _ => Ok(Some(FakeUpdate { app_handle: self.app_handle.clone(), options: self.options.clone() })),
        }
    }
}
