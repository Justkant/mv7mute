use crate::events::{UserEvent, WorkerCommand, WorkerEvent};
use crate::icon::build_icon;
use mv7mute_core::DeviceState;
use std::sync::mpsc::Sender;
use std::thread;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
use tray_icon::{MouseButton, MouseButtonState};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

const MENU_ID_TOGGLE: &str = "toggle";
const MENU_ID_TOGGLE_LOCK: &str = "toggle-lock";
const MENU_ID_REFRESH: &str = "refresh";
const MENU_ID_QUIT: &str = "quit";

#[derive(Clone, Debug)]
pub struct TrayState {
    pub device: Option<DeviceState>,
    pub last_error: Option<String>,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            device: None,
            last_error: Some("Waiting for first refresh".to_string()),
        }
    }
}

struct MenuHandles {
    status_item: MenuItem,
    lock_item: MenuItem,
    toggle_lock_item: MenuItem,
}

pub struct App {
    tray_icon: Option<TrayIcon>,
    menu_handles: Option<MenuHandles>,
    worker_tx: Sender<WorkerCommand>,
    worker_thread: Option<thread::JoinHandle<()>>,
    state: TrayState,
}

impl App {
    pub fn new(worker_tx: Sender<WorkerCommand>, worker_thread: thread::JoinHandle<()>) -> Self {
        Self {
            tray_icon: None,
            menu_handles: None,
            worker_tx,
            worker_thread: Some(worker_thread),
            state: TrayState::default(),
        }
    }

    fn build_tray_icon(&mut self) -> Result<TrayIcon, String> {
        let menu = Menu::new();
        let status_item = MenuItem::new("Status: loading...", false, None);
        let lock_item = MenuItem::new("Lock: unknown", false, None);
        let toggle_item = MenuItem::with_id(MenuId::new(MENU_ID_TOGGLE), "Toggle mute", true, None);
        let toggle_lock_item =
            MenuItem::with_id(MenuId::new(MENU_ID_TOGGLE_LOCK), "Toggle lock", true, None);
        let refresh_item =
            MenuItem::with_id(MenuId::new(MENU_ID_REFRESH), "Refresh now", true, None);
        let quit_item = MenuItem::with_id(MenuId::new(MENU_ID_QUIT), "Quit", true, None);

        menu.append_items(&[
            &status_item,
            &lock_item,
            &PredefinedMenuItem::separator(),
            &toggle_item,
            &toggle_lock_item,
            &refresh_item,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .map_err(|error| error.to_string())?;

        self.menu_handles = Some(MenuHandles {
            status_item,
            lock_item,
            toggle_lock_item,
        });

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("mv7mute tray")
            .with_icon(build_icon(None).map_err(|error| error.to_string())?)
            .with_menu_on_left_click(false)
            .build()
            .map_err(|error| error.to_string())?;

        Ok(tray_icon)
    }

    fn refresh_ui(&self) {
        let Some(menu_handles) = &self.menu_handles else {
            return;
        };

        if let Some(state) = self.state.device {
            menu_handles.status_item.set_text(format!(
                "Status: {}",
                if state.muted { "Muted" } else { "Unmuted" }
            ));
            menu_handles.lock_item.set_text(format!(
                "Lock: {}",
                if state.locked { "Locked" } else { "Unlocked" }
            ));
            menu_handles.toggle_lock_item.set_text(if state.locked {
                "Unlock device"
            } else {
                "Lock device"
            });
        } else if let Some(error) = &self.state.last_error {
            menu_handles.status_item.set_text("Status: unavailable");
            menu_handles.lock_item.set_text(format!("Error: {error}"));
            menu_handles.toggle_lock_item.set_text("Toggle lock");
        }

        if let Some(tray_icon) = &self.tray_icon {
            let _ = tray_icon.set_icon(build_icon(self.state.device).ok());
            let tooltip = self
                .state
                .device
                .map(|state| {
                    format!(
                        "mv7mute: {} ({})",
                        if state.muted { "muted" } else { "unmuted" },
                        if state.locked { "locked" } else { "unlocked" }
                    )
                })
                .or_else(|| self.state.last_error.clone())
                .unwrap_or_else(|| "mv7mute tray".to_string());
            let _ = tray_icon.set_tooltip(Some(tooltip));
        }
    }

    fn request_refresh(&self) {
        let _ = self.worker_tx.send(WorkerCommand::Refresh);
    }

    fn request_toggle(&self) {
        let _ = self.worker_tx.send(WorkerCommand::Toggle);
    }

    fn request_toggle_lock(&self) {
        let _ = self.worker_tx.send(WorkerCommand::ToggleLock);
    }

    fn handle_worker_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::StateUpdated(state) => {
                self.state.device = Some(state);
                self.state.last_error = None;
            }
            WorkerEvent::Error(error) => {
                self.state.device = None;
                self.state.last_error = Some(error);
            }
        }
        self.refresh_ui();
    }

    fn handle_menu_event(&mut self, event_loop: &ActiveEventLoop, event: MenuEvent) {
        match event.id.as_ref() {
            MENU_ID_TOGGLE => self.request_toggle(),
            MENU_ID_TOGGLE_LOCK => self.request_toggle_lock(),
            MENU_ID_REFRESH => self.request_refresh(),
            MENU_ID_QUIT => event_loop.exit(),
            _ => {}
        }
    }

    fn handle_tray_event(&mut self, _event: TrayIconEvent) {
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = _event
        {
            self.request_toggle();
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.worker_tx.send(WorkerCommand::Shutdown);
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init && self.tray_icon.is_none() {
            match self.build_tray_icon() {
                Ok(tray_icon) => {
                    self.tray_icon = Some(tray_icon);
                    self.refresh_ui();
                    self.request_refresh();
                }
                Err(error) => {
                    self.state.last_error = Some(error);
                }
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Tray(event) => self.handle_tray_event(event),
            UserEvent::Menu(event) => self.handle_menu_event(event_loop, event),
            UserEvent::Worker(event) => self.handle_worker_event(event),
        }
    }
}
