use serde::Serialize;

use crate::updater::UpdateStatus;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadResult { 
    pub update_status: UpdateStatus
}