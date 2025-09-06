use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use log::*;
use tauri::{AppHandle, Emitter, Listener};

#[derive(Debug)]
pub enum FlagAction {
    Reset,
    Saved,
    Paused,
    BossOnlyDamage,
    None,
}

pub struct AppListener {
    app_handle: AppHandle,
    reset: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    save: Arc<AtomicBool>,
    boss_only_damage: Arc<AtomicBool>,
    emit_details: Arc<AtomicBool>,
}

impl AppListener {
    pub fn new(app_handle: AppHandle, boss_only_damage_flag: bool) -> Self {

        let reset = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let save = Arc::new(AtomicBool::new(false));
        let boss_only_damage = Arc::new(AtomicBool::new(false));
        let emit_details = Arc::new(AtomicBool::new(false));

        if boss_only_damage_flag {
            boss_only_damage.store(true, Ordering::Relaxed);
            info!("boss only damage enabled")
        }

        let app_handle_clone = app_handle.clone();
        app_handle.listen_any("reset-request", {
            let reset_clone = reset.clone();
            let app_clone = app_handle_clone.clone();
            move |_event| {
                reset_clone.store(true, Ordering::Relaxed);
                info!("resetting meter");
                app_clone.emit("reset-encounter", "").ok();
            }
        });

        app_handle.listen_any("save-request", {
            let save_clone = save.clone();
            let app_clone = app_handle_clone.clone();
            move |_event| {
                save_clone.store(true, Ordering::Relaxed);
                info!("manual saving encounter");
                app_clone.emit("save-encounter", "").ok();
            }
        });

        app_handle.listen_any("pause-request", {
            let pause_clone = pause.clone();
            let app_clone = app_handle_clone.clone();
            move |_event| {
                let prev = pause_clone.fetch_xor(true, Ordering::Relaxed);
                if prev {
                    info!("unpausing meter");
                } else {
                    info!("pausing meter");
                }
                app_clone.emit("pause-encounter", "").ok();
            }
        });

        app_handle.listen_any("boss-only-damage-request", {
            let boss_only_damage = boss_only_damage.clone();
            move |event| {
                let bod = event.payload();
                if bod == "true" {
                    boss_only_damage.store(true, Ordering::Relaxed);
                    info!("boss only damage enabled")
                } else {
                    boss_only_damage.store(false, Ordering::Relaxed);
                    info!("boss only damage disabled")
                }
            }
        });

        app_handle.listen_any("emit-details-request", {
            let emit_clone = emit_details.clone();
            move |_event| {
                let prev = emit_clone.fetch_xor(true, Ordering::Relaxed);
                if prev {
                    info!("stopped sending details");
                } else {
                    info!("sending details");
                }
            }
        });

        Self {
            app_handle,
            reset,
            pause,
            save,
            boss_only_damage,
            emit_details
        }
    }

    pub fn can_emit_details(&self) -> bool {
        self.emit_details.load(Ordering::Relaxed)
    }

    pub fn process_flags(&self) -> FlagAction {

        if self.reset.load(Ordering::Relaxed) {
            self.reset.store(false, Ordering::Relaxed);
            return FlagAction::Reset;
        }

        if self.pause.load(Ordering::Relaxed) {
            return FlagAction::Paused;
        }

        if self.save.load(Ordering::Relaxed) {
            self.save.store(false, Ordering::Relaxed);
            return FlagAction::Saved;
        }

        if self.boss_only_damage.load(Ordering::Relaxed) {
            return FlagAction::BossOnlyDamage;
        }

        FlagAction::None
    }
}