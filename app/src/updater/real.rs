#![allow(dead_code)]

use anyhow::Result;
use async_trait::async_trait;
use tauri::AppHandle;
use tauri_plugin_updater::{Update, Updater, UpdaterExt};

use super::traits::{Updatable, UpdateProvider};

#[derive(Clone)]
pub struct AppUpdate(Update);

impl AppUpdate {
    pub fn new(update: Update) -> Self {
        Self(update)
    }
}

pub struct AppUpdater(AppHandle, Option<Updater>);

impl AppUpdater {
    pub fn new(app_handle: AppHandle) -> Self {
        Self(app_handle, None)
    }
}

#[async_trait]
impl Updatable for AppUpdate {
    async fn download<C, D>(&self, on_chunk: C, on_finish: D) -> Result<Vec<u8>>
    where
        C: FnMut(usize, Option<u64>) + Send + 'static,
        D: FnOnce() + Send + 'static,
    {
        Ok(self.0.download(on_chunk, on_finish).await?)
    }

    fn install(&self, data: Vec<u8>) -> Result<()> {
        self.0.install(data)?;
        Ok(())
    }
}

#[async_trait]
impl UpdateProvider<AppUpdate> for AppUpdater {
    fn setup(&self) -> Result<Self> {
        let updater = self.0.updater()?;

        Ok(Self(self.0.clone(), Some(updater)))
    }

    async fn check(&mut self) -> Result<Option<AppUpdate>> {
        let updater = self.1.take().unwrap();
        updater.check().await.map(|pr | match pr {
            Some(update) => Some(AppUpdate::new(update)),
            None => None,
        }).map_err(Into::into)
    }
}

pub type RealUpdater = AppUpdater;