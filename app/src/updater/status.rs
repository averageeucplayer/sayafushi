#[allow(dead_code)]

use serde::Serialize;

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