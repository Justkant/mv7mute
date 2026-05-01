use mv7mute_core::DeviceState;
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent;

#[derive(Clone, Debug)]
pub enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
    Worker(WorkerEvent),
}

#[derive(Clone, Debug)]
pub enum WorkerEvent {
    StateUpdated(DeviceState),
    Error(String),
}

#[derive(Debug)]
pub enum WorkerCommand {
    Refresh,
    Toggle,
    ToggleLock,
    Shutdown,
}
