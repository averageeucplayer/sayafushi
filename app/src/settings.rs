use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fs::File, path::PathBuf};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub general: GeneralSettings,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut extra = Map::new();

        let shortcuts = serde_json::json!({
            "hideMeter": "Control+ArrowDown",
            "showLogs": "Control+ArrowUp",
            "showLatestEncounter": "",
            "resetSession": "",
            "pauseSession": "",
            "manualSave": "",
            "disableClickthrough": ""
        });
        extra.insert("shortcuts".to_string(), shortcuts);

        let meter = serde_json::json!({
            "bossInfo": true,
            "bossHpBar": true,
            "splitBossHpBar": false,
            "showTimeUntilKill": false,
            "splitPartyBuffs": true,
            "showClassColors": true,
            "profileShortcut": false,
            "damage": false,
            "dps": true,
            "damagePercent": true,
            "deathTime": false,
            "incapacitatedTime": false,
            "critRate": true,
            "critDmg": false,
            "frontAtk": true,
            "backAtk": true,
            "counters": false,
            "pinSelfParty": false,
            "positionalDmgPercent": true,
            "percentBuffBySup": true,
            "percentIdentityBySup": true,
            "percentBrand": true,
            "percentHatBySup": true,
            "breakdown": {
                "damage": true,
                "dps": true,
                "damagePercent": true,
                "critRate": true,
                "critDmg": false,
                "frontAtk": true,
                "backAtk": true,
                "avgDamage": false,
                "maxDamage": true,
                "casts": true,
                "cpm": true,
                "hits": false,
                "hpm": false,
                "percentBuffBySup": false,
                "percentIdentityBySup": false,
                "percentBrand": false,
                "percentHatBySup": false
            }
        });

        extra.insert("meter".to_string(), meter);

        let logs = serde_json::json!({
            "abbreviateHeader": false,
            "splitPartyDamage": true,
            "splitPartyBuffs": true,
            "profileShortcut": true,
            "damage": true,
            "dps": true,
            "damagePercent": true,
            "deathTime": true,
            "incapacitatedTime": true,
            "critRate": true,
            "critDmg": false,
            "frontAtk": true,
            "backAtk": true,
            "counters": true,
            "minEncounterDuration": 30,
            "positionalDmgPercent": true,
            "percentBuffBySup": true,
            "percentIdentityBySup": true,
            "percentHatBySup": true,
            "percentBrand": true,
            "breakdown": {
                "damage": true,
                "dps": true,
                "damagePercent": true,
                "critRate": true,
                "adjustedCritRate": true,
                "critDmg": false,
                "frontAtk": true,
                "backAtk": true,
                "avgDamage": true,
                "maxDamage": true,
                "casts": true,
                "cpm": true,
                "hits": true,
                "hpm": true,
                "percentBuffBySup": false,
                "percentIdentityBySup": false,
                "percentBrand": false,
                "percentHatBySup": false
            }
        });
        extra.insert("logs".to_string(), logs);

        Settings { 
            general: GeneralSettings::default(),
            extra
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneralSettings {
    pub start_loa_on_start: bool,
    pub low_performance_mode: bool,
    #[serde(default = "default_true")]
    pub auto_iface: bool,
    pub port: u16,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub boss_only_damage: bool,
    #[serde(default = "default_true")]
    pub hide_meter_on_start: bool,
    pub hide_logs_on_start: bool,
    pub mini: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_true() -> bool {
    true
}


pub struct SettingsManager(PathBuf);

impl SettingsManager {
    pub fn new(path: PathBuf) -> Result<Self> {

        if !path.exists() {
            let writer = File::create(&path)?;
            let settings = Settings::default();
            serde_json::to_writer_pretty(writer, &settings)?;
        }

        Ok(Self(path))
    }

    pub fn read(&self) -> Result<Settings> {
        let reader = File::open(&self.0)?;
        let settings = serde_json::from_reader(reader)?;

        Ok(settings)
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        let writer = File::create(&self.0)?;
        serde_json::to_writer_pretty(writer, settings)?;

        Ok(())
    }
}