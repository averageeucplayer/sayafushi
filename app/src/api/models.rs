use serde::{Deserialize, Serialize};

use crate::models::ArkPassiveData;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCharacterInfoArgs {
    pub client_id: String,
    pub version: String,
    pub region: String,
    pub raid_name: String,
    pub boss: String,
    pub characters: Vec<String>,
    pub difficulty: Option<String>,
    pub cleared: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InspectInfo {
    pub combat_power: Option<CombatPower>,
    pub ark_passive_enabled: bool,
    pub ark_passive_data: Option<ArkPassiveData>,
    pub engravings: Option<Vec<u32>>,
    pub gems: Option<Vec<GemData>>,
    pub loadout_snapshot: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombatPower {
    // 1 for dps, 2 for support
    pub id: u32,
    pub score: f32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GemData {
    pub tier: u8,
    pub skill_id: u32,
    pub gem_type: u8,
    pub value: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Engraving {
    pub id: u32,
    pub level: u8,
}
