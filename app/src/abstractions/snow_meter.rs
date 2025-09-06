use std::sync::mpsc::Receiver;

use meter_core::packets::opcodes::Pkt;
use meter_core::packets::structures::SkillDamageEvent;
use meter_core::start_capture;
use meter_core::decryption::DamageEncryptionHandler as MeterCoreDamageEncryptionHandler;
use anyhow::Result;
use crate::abstractions::{DamageEncryptionHandler, PacketReceiver, PacketSource};

pub use meter_core::*;

pub struct SnowDamageEncryptionHandler(Option<MeterCoreDamageEncryptionHandler>);

pub struct DefaultReceiver(Receiver<(Pkt, Vec<u8>)>);

impl PacketReceiver for DefaultReceiver {
    fn recv(&mut self) -> Result<(Pkt, Vec<u8>)> {
        Ok(self.0.recv()?)
    }
}

pub struct WindivertPacketCapture {
    region_file_path: String
}

impl PacketSource<DefaultReceiver> for WindivertPacketCapture {
    fn start(&self, port: u16) -> Result<DefaultReceiver> {
        let receiver = start_capture(port, self.region_file_path.clone())?;
        Ok(DefaultReceiver(receiver))
    }
}

impl DamageEncryptionHandler for SnowDamageEncryptionHandler {
    fn start(&mut self) -> Result<()> {
        let handler = MeterCoreDamageEncryptionHandler::new();
        let handler = handler.start()?;
        self.0 = Some(handler);
        Ok(())
    }

    fn decrypt_damage_event(&self, skill_damage_event: &mut SkillDamageEvent) -> bool {
        let handler = self.0.as_ref().expect("DamageEncryptionHandler is not initialized");

        handler.decrypt_damage_event(skill_damage_event)
    }

    fn update_zone_instance_id(&mut self, zone_instance_id: u32) {
        let handler = self.0.as_mut().expect("DamageEncryptionHandler is not initialized");

        handler.update_zone_instance_id(zone_instance_id);
    }
}


impl WindivertPacketCapture {
    pub fn new(region_file_path: String) -> Self {
        Self {
            region_file_path
        }
    }
}

impl SnowDamageEncryptionHandler {
    pub fn new() -> Self {
        Self(None)
    }
}