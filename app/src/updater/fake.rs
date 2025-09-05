use std::{fs::File, io::{BufReader, Read}, marker::PhantomData, path::PathBuf, sync::{Arc, Mutex}, time::Duration};
use anyhow::Result;
use tauri::{async_runtime::spawn, AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;
use tokio::time::sleep;

use super::status::UpdateStatus;
use super::manager::UpdateManagerImpl;

/// Configurable options for the fake updater.
#[derive(Clone)]
pub enum FakeUpdateOptions {
    Latest,
    /// Simulate a download with synthetic data.
    Synthetic {
        /// Pretend the HTTP server provides `Content-Length` in the header.
        with_total_header: bool,
        /// Total size of the fake update (in bytes).
        total_size: u64,
        /// Number of chunks to split the download into.
        iterations: usize,
        /// Delay between chunks.
        delay: Duration,
    },

    /// Stream from an actual binary file on disk.
    Binary {
        /// Path to the binary to stream.
        path: PathBuf,
        /// Pretend the HTTP server provides `Content-Length` in the header.
        with_total_header: bool,
        /// Delay between chunks (simulating network latency).
        delay: Duration,
    },
}

impl Default for FakeUpdateOptions {
    fn default() -> Self {
        FakeUpdateOptions::Synthetic {
            with_total_header: true,
            total_size: 200 * 1024 * 1024, // 200 MB
            iterations: 25,
            delay: Duration::from_millis(100),
        }
    }
}

#[derive(Clone)]
pub struct FakeUpdater {
    options: FakeUpdateOptions
}

#[derive(Clone)]
pub struct FakeUpdate {
    pub app_handle: AppHandle, 
    pub options: FakeUpdateOptions,
}

impl FakeUpdate {
    pub fn new(app_handle: AppHandle, options: FakeUpdateOptions) -> Self {
        Self { 
            app_handle,
            options
        }
    }

    pub async fn download<C, D>(&self, mut on_chunk: C, on_finish: D) -> Result<Vec<u8>>
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
                let total_size = *total_size;
                let iterations = *iterations;
                let chunk_size = (total_size / iterations as u64) as usize;
                let last_chunk_size = (total_size % iterations as u64) as usize;

                let mut data = Vec::with_capacity(total_size as usize);

                for i in 0..iterations {
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
            _ => unreachable!()
        }
    }

    pub fn install(&self, _data: Vec<u8>) -> Result<()> {

        if let FakeUpdateOptions::Binary { path, .. } = &self.options {
            self.app_handle.opener().open_path(path.to_string_lossy(), None::<String>)?;
        }

        Ok(())
    }
}

impl UpdateManagerImpl<FakeUpdater> {
    pub fn new(app_handle: AppHandle, options: FakeUpdateOptions) -> Self {
        Self {
            app_handle,
            handle: Mutex::new(None),
            status: Arc::new(Mutex::new(UpdateStatus::Checking)),
            _phantom: PhantomData,
            fake_options: Some(options)
        }
    }

    pub fn check_updates(&mut self) {
        let status = self.status.clone();
        let app_handle = self.app_handle.clone();
        let options = self.fake_options.take().unwrap();

        let handle = spawn(async move {

            if let FakeUpdateOptions::Latest = options {
                let mut st = status.lock().unwrap();
                *st = UpdateStatus::Latest;
                app_handle.emit("on-update", UpdateStatus::Latest).unwrap();
                return;
            }

            let update = FakeUpdate::new(app_handle.clone(), options.clone());
            let status_chunk = status.clone();
            let status_finish = status.clone();
            let app_handle_chunk = app_handle.clone();
            let app_handle_finish = app_handle.clone();
            let mut total = 0;

            let data = update.download(
                move |chunk, length| {
                    total += chunk;
                    let mut st = status_chunk.lock().unwrap();
                    let update_status = UpdateStatus::Downloading { chunk: total, length };
                    *st = update_status.clone();
                    app_handle_chunk.emit("on-update", update_status).unwrap();
                },
                move || {
                    let mut st = status_finish.lock().unwrap();
                    *st = UpdateStatus::Downloaded;
                    app_handle_finish.emit("on-update", UpdateStatus::Downloaded).unwrap();
                },
            ).await.unwrap();

            sleep(Duration::from_millis(500)).await;

            if let FakeUpdateOptions::Synthetic { .. } = &options {
                let mut st = status.lock().unwrap();
                *st = UpdateStatus::Latest;
                app_handle.emit("on-update", UpdateStatus::Latest).unwrap();
            }

            update.install(data).unwrap();
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
