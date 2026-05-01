use mv7mute_core::DeviceState;
use tray_icon::{BadIcon, Icon};

pub fn build_icon(state: Option<DeviceState>) -> Result<Icon, BadIcon> {
    let (background, accent) = match state {
        Some(DeviceState { muted: true, .. }) => {
            ([0x8E, 0x1B, 0x1B, 0xFF], [0xF6, 0xD7, 0xD7, 0xFF])
        }
        Some(DeviceState { muted: false, .. }) => {
            ([0x14, 0x6C, 0x43, 0xFF], [0xD7, 0xF5, 0xE3, 0xFF])
        }
        None => ([0x5A, 0x5F, 0x66, 0xFF], [0xEE, 0xF1, 0xF4, 0xFF]),
    };

    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let center = (size as f32 - 1.0) / 2.0;
    let radius = size as f32 * 0.34;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            let pixel = if distance <= radius {
                accent
            } else {
                background
            };
            rgba.extend_from_slice(&pixel);
        }
    }

    Icon::from_rgba(rgba, size, size)
}
