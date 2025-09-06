use anyhow::Result;
use std::{path::PathBuf, thread::JoinHandle};
use log::*;
use tauri::{AppHandle, Manager};

use crate::{data::AssetPreloader, settings::Settings, updater::UpdateManager};

macro_rules! background_worker {
    ($name:expr, $args:expr, $body:expr) => {
        std::thread::Builder::new()
            .name($name.to_string())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move { $body($args).await })
            })
    };
}

pub struct BackgroundWorkerArgs {
    pub app_handle: AppHandle,
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
      
        let handle = background_worker!("background-worker", args, Self::inner)?;

        self.0 = Some(handle);

        Ok(())
    }

    async fn inner(args: BackgroundWorkerArgs) -> Result<()> {
        let BackgroundWorkerArgs {
            app_handle,
            port,
            region_file_path,
            settings,
            version
        } = args;

        let asset_preloader = app_handle.state::<AssetPreloader>();
        info!("waiting for assets to load");
        asset_preloader.wait().unwrap();

        info!("waiting for update manager");
        let update_manager = app_handle.state::<UpdateManager>();
        update_manager.wait().await.unwrap();

        info!("listening on port: {}", port);
        
        #[cfg(feature = "meter-core")]
        {
            use std::marker::PhantomData;

            use crate::{abstractions::{DefaultRegionAccessor, SnowDamageEncryptionHandler, WindivertPacketCapture}, api::{SnowHeartbeatApi, SnowStatsApi}, live::{self, StartArgs}};

            let heartbeat_api = Box::new(SnowHeartbeatApi::new(settings.env.hearbeat_api_url.clone()));
            let region_accessor = Box::new(DefaultRegionAccessor::new(region_file_path.clone().into()));
            let packet_source = WindivertPacketCapture::new(region_file_path.display().to_string());
            let damage_handler = SnowDamageEncryptionHandler::new();
            let stats_api = {
                use crate::local::LocalPlayerRepository;

                let local: tauri::State<'_, LocalPlayerRepository> = app_handle.state::<LocalPlayerRepository>();
                let local_info = local.read()?;
                Box::new(SnowStatsApi::new(settings.env.stats_api_url.clone(), local_info.client_id))
            };

            app_handle.manage(stats_api);
            
            let args = StartArgs  {
                app_handle,
                port,
                settings,
                version,
                heartbeat_api,
                packet_source,
                region_accessor,
                damage_handler,
                _marker: PhantomData,
            };

            live::start(args).expect("An error occurred whilst running packet processor");
        }

        #[cfg(feature = "meter-core-fake")]
        {
            use std::marker::PhantomData;

            use crate::{abstractions::{DefaultDamageEncryptionHandler, FakePacketSource, FakeRegionAccessor}, api::FakeHeartbeatApi, live::{self, StartArgs}};

            let heartbeat_api = Box::new(FakeHeartbeatApi::new());
            let region_accessor = Box::new(FakeRegionAccessor::new("EUC".into()));
            let packet_source = FakePacketSource::new();
            let damage_handler = DefaultDamageEncryptionHandler::new();

            let stats_api = Box::new(SnowStatsApi::new(settings.env.stats_api_url));

            app_handle.manage(stats_api);

            let args = StartArgs {
                app_handle,
                port,
                settings,
                version,
                heartbeat_api,
                packet_source,
                region_accessor,
                damage_handler,
                _marker: PhantomData,
            };

            live::start(args).expect("An error occurred whilst running packet processor");;
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|handle| !handle.is_finished())
    }
}