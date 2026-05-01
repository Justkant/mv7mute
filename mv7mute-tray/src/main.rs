#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod events;
mod icon;
mod worker;

use app::App;
use events::UserEvent;
use single_instance::SingleInstance;
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use worker::spawn_worker;

const SINGLE_INSTANCE_ID: &str = "mv7mute-tray-main";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = SingleInstance::new(SINGLE_INSTANCE_ID)?;
    if !instance.is_single() {
        eprintln!("mv7mute-tray is already running");
        return Ok(());
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();

    TrayIconEvent::set_event_handler(Some({
        let proxy = proxy.clone();
        move |event| {
            let _ = proxy.send_event(UserEvent::Tray(event));
        }
    }));

    MenuEvent::set_event_handler(Some({
        let proxy = proxy.clone();
        move |event| {
            let _ = proxy.send_event(UserEvent::Menu(event));
        }
    }));

    let (worker_tx, worker_thread) = spawn_worker(proxy);
    let mut app = App::new(worker_tx, worker_thread);
    event_loop.run_app(&mut app)?;
    drop(instance);
    Ok(())
}
