use crate::live::encounter_state::EncounterState;
use crate::live::entity_tracker::EntityTracker;
use crate::live::party_tracker::PartyTracker;
use crate::live::utils::update_party;
use crate::models::*;
use log::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub struct SendToUiArgs<'a> {
    pub state: &'a mut EncounterState,
    pub party_freeze: bool,
    pub party_cache: &'a mut Option<Vec<Vec<String>>>,
    pub party_tracker: Rc<RefCell<PartyTracker>>,
    pub entity_tracker: &'a EntityTracker
}

pub struct AppSender {
    app_handle: AppHandle,
    last_update: Instant,
    duration: Duration,
    last_party_update: Instant,
    party_duration: Duration,
}

impl AppSender {
    pub fn new(app_handle: AppHandle, low_performance_mode: bool) -> Self {
        let last_update = Instant::now();
        let mut duration = Duration::from_millis(200);
        let last_party_update = Instant::now();
        let party_duration = Duration::from_millis(2000);

        if low_performance_mode {
            duration = Duration::from_millis(1500);
            info!("low performance mode enabled")
        }

        Self {
            app_handle,
            last_update,
            duration,
            last_party_update,
            party_duration
        }
    }

    pub fn send_to_ui(&mut self, args: SendToUiArgs) {

        let SendToUiArgs {
            state,
            party_freeze,
            party_cache,
            entity_tracker,
            party_tracker
        } = args;
        
        let can_send = self.last_update.elapsed() >= self.duration || state.resetting || state.boss_dead_update;

        if !can_send {
            return;
        }

        let boss_dead = state.boss_dead_update;
        
        if state.boss_dead_update {
            state.boss_dead_update = false;
        }

        let mut clone = state.encounter.clone();
        let damage_valid = state.damage_is_valid;
        let app_handle = self.app_handle.clone();

        let party_info: Option<Vec<Vec<String>>> =
            if self.last_party_update.elapsed() >= self.party_duration && !party_freeze {
                self.last_party_update = Instant::now();

                // use cache if available
                // otherwise get party info
                party_cache.clone().or_else(|| {
                    let party = update_party(&party_tracker, &entity_tracker);
                    if party.len() > 1 {
                        if party.iter().all(|p| p.len() == 4) {
                            *party_cache = Some(party.clone());
                        }
                        Some(party)
                    } else {
                        None
                    }
                })
            } else {
                None
            };

        tokio::task::spawn(async move {
            if !clone.current_boss_name.is_empty() {
                let current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                if let Some(mut current_boss) = current_boss {
                    if boss_dead {
                        current_boss.is_dead = true;
                        current_boss.current_hp = 0;
                    }
                    clone.current_boss = Some(current_boss);
                } else {
                    clone.current_boss_name = String::new();
                }
            }
            clone.entities.retain(|_, e| {
                ((e.entity_type == EntityType::Player && e.class_id > 0)
                    || e.entity_type == EntityType::Esther
                    || e.entity_type == EntityType::Boss)
                    && e.damage_stats.damage_dealt > 0
            });

            if !clone.entities.is_empty() {
                if !damage_valid {
                    app_handle
                        .emit("invalid-damage", "")
                        .expect("failed to emit invalid-damage");
                } else {
                    app_handle
                        .emit("encounter-update", Some(clone))
                        .expect("failed to emit encounter-update");

                    if party_info.is_some() {
                        app_handle
                            .emit("party-update", party_info)
                            .expect("failed to emit party-update");
                    }
                }
            }
        });

        self.last_update = Instant::now();
    }
}