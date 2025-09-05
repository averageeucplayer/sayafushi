use anyhow::Result;
use std::{path::PathBuf, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self, JoinHandle}};
use log::*;
use tauri::{AppHandle, Manager};
use tokio::runtime::Runtime;

use crate::{data::AssetPreloader, settings::Settings, updater::UpdateManager};

pub struct BackgroundWorkerArgs {
    pub app: AppHandle,
    pub port: u16,
    pub region_file_path: PathBuf,
    pub settings: Settings,
    pub version: String
}

pub struct BackgroundWorker(Option<JoinHandle<Result<()>>>);

impl BackgroundWorker {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn start(&mut self, args: BackgroundWorkerArgs) -> Result<()> {
      
        let builder = thread::Builder::new().name("background-worker".to_string());

        let handle = builder.spawn(move || Self::inner(args))?;

        self.0 = Some(handle);

        Ok(())
    }

    fn inner(args: BackgroundWorkerArgs) -> Result<()> {
        let BackgroundWorkerArgs {
            app,
            port,
            region_file_path,
            settings,
            version
        } = args;

        // only start listening when there's no update, otherwise unable to remove driver
        // while !update_checked.load(Ordering::Relaxed) {
        //     std::thread::sleep(std::time::Duration::from_millis(100));
        // }

        let rt = Runtime::new()?;
        
        rt.block_on(async move {
            let asset_preloader = app.state::<AssetPreloader>();
            info!("waiting for assets to load");
            asset_preloader.wait().unwrap();

            info!("waiting for update manager");
            let update_manager = app.state::<UpdateManager>();
            update_manager.wait().await.unwrap();

            info!("listening on port: {}", port);
            
            #[cfg(feature = "meter-core")]
            {
                use crate::live;

                live::start(app, port, Some(settings)).map_err(|e| {
                    error!("unexpected error occurred in parser: {e}");
                })
            }

            #[cfg(feature = "meter-core-fake")]
            {
                use crate::live;

                live::start(app, port, Some(settings)).map_err(|e| {
                    error!("unexpected error occurred in parser: {e}");
                })
            }
        }).unwrap();

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|handle| !handle.is_finished())
    }
}