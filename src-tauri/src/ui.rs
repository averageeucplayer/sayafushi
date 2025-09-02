use std::str::FromStr;

use log::*;
use anyhow::{anyhow, Result};
use strum::EnumProperty;
use strum_macros::{AsRefStr, EnumProperty, EnumString};
use tauri::{async_runtime, menu::{Menu, MenuBuilder, MenuEvent}, tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconEvent}, AppHandle, Manager, Runtime, Window, WindowEvent};
use tauri_plugin_window_state::AppHandleExt;

use crate::{constants::*, extensions::{AppHandleExtensions, WindowExtensions}, settings::SettingsManager, shell::ShellManager};

#[derive(Debug, EnumString, EnumProperty, AsRefStr)]
#[strum(serialize_all = "kebab_case")]
pub enum TrayCommand {
    #[strum(props(label = "Show Logs"))]
    ShowLogs,

    #[strum(props(label = "Show Meter"))]
    ShowMeter,

    #[strum(props(label = "Hide Meter"))]
    Hide,

    #[strum(props(label = "Start Lost Ark"))]
    StartLoa,

    #[strum(props(label = "Reset Window"))]
    Reset,

    #[strum(props(label = "Quit"))]
    Quit,
}

pub struct LoaMenuBuilder<'a, R: Runtime>(
    MenuBuilder<'a, R, AppHandle<R>>
);

impl<'a, R: Runtime> LoaMenuBuilder<'a, R> {
    pub fn new(app: &'a AppHandle<R>) -> Self {
        Self(MenuBuilder::new(app))
    }

    pub fn command(mut self, cmd: TrayCommand) -> Self {
        self.0 = self.0.text(cmd.as_ref(), cmd.get_str("label").unwrap());
        self
    }

    pub fn separator(mut self) -> Self {
        self.0 = self.0.separator();
        self
    }

    pub fn build(self) -> tauri::Result<Menu<R>> {
        self.0.build()
    }
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
     let menu = LoaMenuBuilder::new(app)
        .command(TrayCommand::ShowLogs)
        .separator()
        .command(TrayCommand::ShowMeter)
        .command(TrayCommand::Hide)
        .separator()
        .command(TrayCommand::StartLoa)
        .separator()
        .command(TrayCommand::Reset)
        .separator()
        .command(TrayCommand::Quit)
        .build()?;

    let tray = app.tray_by_id(METER_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find main window"))?;
    tray.set_menu(Some(menu))?;
    tray.on_menu_event(on_menu_event);
    tray.on_tray_icon_event(on_tray_icon_event);

    Ok(())
}

pub fn on_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
     {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app_handle = tray.app_handle();
            if let Some(meter) = app_handle.get_meter_window() {
                meter.restore_and_focus();
            }
        }
    }
}

pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    if let Err(err) = on_menu_event_inner(app, event) {
        error!("An error occurred whilst handling menu event {}", err);
    }
}

pub fn on_menu_event_inner(app_handle: &AppHandle, event: MenuEvent) -> Result<()> {
    let menu_item_id = event.id().0.as_str();
    let settings_manager = app_handle.state::<SettingsManager>();

    match TrayCommand::from_str(menu_item_id)? {
        TrayCommand::Quit => {
            app_handle.save_window_state(WINDOW_STATE_FLAGS)?;
            
            let shell_manager = app_handle.state::<ShellManager>();
            async_runtime::block_on(async {
                shell_manager.unload_driver().await;
            });
        
            app_handle.exit(0);
        }
        TrayCommand::Hide => {
            if let Some(meter) = app_handle.get_meter_window() {
                meter.hide()?;
            }

            if let Some(mini) = app_handle.get_mini_window() {
                mini.hide()?;
            }
        }
        TrayCommand::ShowMeter => {
            let settings_manager = app_handle.state::<SettingsManager>();
            let settings = settings_manager.read()?;
            let window = app_handle.get_window(settings.general.mini);
            window.restore_and_focus();
        }
        TrayCommand::Reset => {
            let settings = settings_manager.read()?;

            if settings.general.mini {
                if let Some(mini) = app_handle.get_mini_window() {
                    mini.set_size(DEFAULT_MINI_METER_WINDOW_SIZE)?;
                    mini.set_position(WINDOW_POSITION)?;
                    mini.restore_and_focus();
                }

                return Ok(())
            }

            if let Some(meter) = app_handle.get_meter_window() {
                meter.set_size(DEFAULT_METER_WINDOW_SIZE)?;
                meter.set_position(WINDOW_POSITION)?;
                meter.restore_and_focus();
            }
        }
        TrayCommand::ShowLogs => {
            if let Some(logs) = app_handle.get_logs_window() {
                logs.show().unwrap();
                logs.unminimize().unwrap();
            }
        }
        TrayCommand::StartLoa => {
            let shell_manager = app_handle.state::<ShellManager>();
            shell_manager.start_loa_process();
        }
        _ => {}
    }

    Ok(())
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let label = window.label();
    on_window_event_inner(label, window, event).expect("An error occurred whilst handling window event");
}

pub fn on_window_event_inner(label: &str, window: &Window, event: &WindowEvent) -> Result<()> {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();

            if label == LOGS_WINDOW_LABEL {
                window.hide()?;

                return Ok(())
            }

            let app_handle = window.app_handle();
            let meter_window = app_handle.get_meter_window().unwrap();
            let logs_window = app_handle.get_logs_window().unwrap();

            if logs_window.is_minimized()? {
                logs_window.unminimize()?;
            }

            if meter_window.is_minimized()? {
                meter_window.unminimize()?;
            } 

            let shell_manager = app_handle.state::<ShellManager>();
            async_runtime::block_on(async {
                shell_manager.unload_driver().await;
            });

            app_handle.exit(0);

            Ok(())
        },
        WindowEvent::Focused(focused) => {
            if *focused {
                return Ok(())
            }

            let app_handle = window.app_handle();
            app_handle.save_window_state(WINDOW_STATE_FLAGS)?;

            Ok(())
        },
        _ => Ok(()),
    }
}
