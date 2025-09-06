mod encounter_state;
mod entity_tracker;
mod id_tracker;
pub mod party_tracker;
pub mod skill_tracker;
pub mod status_tracker;
pub mod utils;
mod handler;
mod listener;
mod sender;

use crate::abstractions::{DamageEncryptionHandler, PacketReceiver, PacketSource, RegionAcessor};
use crate::api::{HeartbeatApi, HeartbeatSendArgs};
use crate::live::encounter_state::EncounterState;
use crate::live::entity_tracker::EntityTracker;
use crate::live::handler::{handle, HandleArgs};
use crate::live::id_tracker::IdTracker;
use crate::live::listener::AppListener;
use crate::live::party_tracker::PartyTracker;
use crate::live::sender::{AppSender, SendToUiArgs};
use crate::live::status_tracker::StatusTracker;
use crate::live::utils::update_party;
use crate::local::LocalPlayerRepository;
use crate::settings::Settings;
use anyhow::Result;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Instant;
use tauri::{AppHandle, Manager};

pub struct StartArgs<PR: PacketReceiver, PC: PacketSource<PR>, DH: DamageEncryptionHandler> {
    pub packet_source: PC,
    pub app_handle: AppHandle,
    pub port: u16,
    pub settings: Settings,
    pub version: String,
    pub heartbeat_api: Box<dyn HeartbeatApi>,
    pub region_accessor: Box<dyn RegionAcessor>,
    pub damage_handler: DH,
    pub _marker: PhantomData<PR>,
}

pub fn start<PR: PacketReceiver, PC: PacketSource<PR>, DH: DamageEncryptionHandler>(args: StartArgs<PR, PC, DH>) -> Result<()> {

    let StartArgs {
        packet_source,
        app_handle,
        port,
        settings,
        version,
        mut heartbeat_api,
        region_accessor,
        mut damage_handler,
        ..
    } = args;

    let id_tracker: Rc<RefCell<IdTracker>> = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker: Rc<RefCell<PartyTracker>> = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker: Rc<RefCell<StatusTracker>> = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker: EntityTracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );

    let local: tauri::State<'_, LocalPlayerRepository> = app_handle.state::<LocalPlayerRepository>();
    let mut local_info = local.read()?;

    let mut state = EncounterState::new(
        version.clone(),
        local_info.client_id.clone(),
        app_handle.clone());

    let mut packet_receiver = packet_source.start(port)?;

    damage_handler.start()?;

    let mut raid_end_cd = Instant::now();

    let listener = AppListener::new(app_handle.clone(), settings.general.boss_only_damage);
    let mut sender = AppSender::new(app_handle.clone(), settings.general.low_performance_mode);

    let region = region_accessor.get();
    state.region = region.clone();
    state.encounter.region = region;

    let mut party_freeze = false;
    let mut party_cache: Option<Vec<Vec<String>>> = None;

    while let Ok((op, data)) = packet_receiver.recv() {
        let action = listener.process_flags();

        match action {
            listener::FlagAction::Reset => state.soft_reset(true),
            listener::FlagAction::Saved => {
                state.party_info = update_party(&party_tracker, &entity_tracker);
                state.save_to_db(true);
                state.saved = true;
                state.resetting = true;
            },
            listener::FlagAction::Paused => continue,
            listener::FlagAction::BossOnlyDamage => {
                if state.boss_only_damage {
                    state.boss_only_damage = false;
                    state.encounter.boss_only_damage = false;
                }   
                else {
                    state.boss_only_damage = true;
                } 
            },
            _ => {},
        }

        let args = HandleArgs {
            data: &data,
            op,
            state: &mut state,
            id_tracker: id_tracker.clone(),
            party_tracker: party_tracker.clone(),
            status_tracker: status_tracker.clone(),
            entity_tracker: &mut entity_tracker,
            app_handle: &app_handle,
            damage_handler: &mut damage_handler,
            can_emit_details: listener.can_emit_details(),
            local_info: &mut local_info,
            local: &local,
            party_cache: &mut party_cache,
            party_freeze: &mut party_freeze,
            raid_end_cd: &mut raid_end_cd,
            region_accessor: &region_accessor
        };

        handle(args);

        let args = SendToUiArgs {
            state: &mut state,
            party_cache: &mut party_cache,
            party_freeze,
            party_tracker: party_tracker.clone(),
            entity_tracker: &entity_tracker,
        };

        sender.send_to_ui(args);

        if state.resetting {
            state.soft_reset(true);
            state.resetting = false;
            state.saved = false;
            party_freeze = false;
            party_cache = None;
        }

        if let Some(region) = state.region.as_ref() && heartbeat_api.can_send() {
            let args = HeartbeatSendArgs {
                id: &local_info.client_id,
                region,
                version: &version
            };

            heartbeat_api.send(args);
        }
    }

    Ok(())
}