#![allow(dead_code)]

use async_trait::async_trait;
use hashbrown::HashMap;
use log::*;
use reqwest::Client;

use crate::api::*;

#[async_trait]
pub trait StatsApi: Send + Sync {
    async fn get_character_info(&self, args: GetCharacterInfoArgs) -> Option<HashMap<String, InspectInfo>>;
}

#[derive(Clone)]
pub struct SnowStatsApi {
    base_url: String,
    client_id: String,
    client: Client,
}

#[async_trait]
impl StatsApi for SnowStatsApi {

    async fn get_character_info(&self, args: GetCharacterInfoArgs) -> Option<HashMap<String, InspectInfo>> {

        let url = format!("{}/inspect", self.base_url);
        let result = self.client.post(url).json(&args).send().await;

        match result {
            Ok(res) => match res.json::<HashMap<String, InspectInfo>>().await {
                Ok(data) => {
                    info!("received player stats");
                    Some(data)
                }
                Err(err) => {
                    warn!("failed to parse player stats: {:?}", err);
                    None
                }
            },
            Err(err) => {
                warn!("failed to get inspect data: {:?}", err);
                None
            }
        }
    }
}

impl SnowStatsApi {
    pub fn new(base_url: String, client_id: String) -> Self {
        Self {
            base_url,
            client_id,
            client: Client::new(),
        }
    }
}

pub struct FakeStatsApi;

#[async_trait]
impl StatsApi for FakeStatsApi {

    async fn get_character_info(&self, args: GetCharacterInfoArgs) -> Option<HashMap<String, InspectInfo>> {
        info!("attempt to get character info with args {:?}", args);
        None
    }
}

impl FakeStatsApi {
    pub fn new() -> Self {
        Self
    }
}