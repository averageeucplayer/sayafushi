use std::time::{Duration, Instant};

use log::*;
use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatSendArgs<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub region: &'a str
}

pub trait HeartbeatApi {
    fn can_send(&self) -> bool;
    fn send(&mut self, args: HeartbeatSendArgs);
}

pub struct SnowHeartbeatApi {
    base_url: String,
    client: Client,
    last_heartbeat: Instant,
    heartbeat_duration: Duration
}

impl HeartbeatApi for SnowHeartbeatApi {
    fn can_send(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.heartbeat_duration
    }

    fn send(&mut self, args: HeartbeatSendArgs) {

        let client = self.client.clone();
        let url = format!("{}/analytics/heartbeat", self.base_url);
        let json = serde_json::to_value(args).unwrap();

        tokio::task::spawn(async move {
            match client
                .post(url)
                .json(&json)
                .send()
                .await
            {
                Ok(_) => {
                    info!("sent heartbeat");
                }
                Err(e) => {
                    warn!("failed to send heartbeat: {:?}", e);
                }
            }
        });
    
        self.last_heartbeat = Instant::now();
    }
}

impl SnowHeartbeatApi {
    pub fn new(base_url: String) -> Self {
        let client = Client::new();
        let last_heartbeat = Instant::now();
        let heartbeat_duration = Duration::from_secs(60 * 15);
        Self {
            base_url,
            client,
            last_heartbeat,
            heartbeat_duration
        }
    }
}

pub struct FakeHeartbeatApi {
    last_heartbeat: Instant,
    heartbeat_duration: Duration
}

impl HeartbeatApi for FakeHeartbeatApi {
    fn can_send(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.heartbeat_duration
    }

    fn send(&mut self, args: HeartbeatSendArgs) {
        info!("heartbeat client_id: {} region: {:?} version: {}", args.id, args.region, args.version);
        self.last_heartbeat = Instant::now();
    }
}

impl FakeHeartbeatApi {
    pub fn new() -> Self {
        let last_heartbeat = Instant::now();
        let heartbeat_duration = Duration::from_secs(60 * 15);

        Self {
            last_heartbeat,
            heartbeat_duration
        }
    }
}