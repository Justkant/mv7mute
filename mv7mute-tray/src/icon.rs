use image::imageops::FilterType;
use mv7mute_core::DeviceState;
use std::sync::OnceLock;
use tray_icon::{BadIcon, Icon};

const TRAY_ICON_SIZE: u32 = 32;
const UNMUTED_ICON_PNG: &[u8] = include_bytes!("../assets/microphone-solid.png");
const MUTED_ICON_PNG: &[u8] = include_bytes!("../assets/microphone-slash-solid.png");
const COLOR_UNMUTED: [u8; 3] = [0x14, 0x6C, 0x43];
const COLOR_MUTED: [u8; 3] = [0x8E, 0x1B, 0x1B];
const COLOR_UNKNOWN: [u8; 3] = [0x5A, 0x5F, 0x66];

struct IconImage {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

struct LoadedIcons {
    muted: IconImage,
    unmuted: IconImage,
    unknown: IconImage,
}

static ICONS: OnceLock<Result<LoadedIcons, String>> = OnceLock::new();

pub fn build_icon(state: Option<DeviceState>) -> Result<Icon, BadIcon> {
    let icons = ICONS.get_or_init(load_icons);
    let icon = match icons {
        Ok(icons) => match state {
            Some(DeviceState { muted: true, .. }) => &icons.muted,
            Some(DeviceState { muted: false, .. }) => &icons.unmuted,
            None => &icons.unknown,
        },
        Err(_) => {
            let mut fallback = Vec::with_capacity((TRAY_ICON_SIZE * TRAY_ICON_SIZE * 4) as usize);
            for _ in 0..(TRAY_ICON_SIZE * TRAY_ICON_SIZE) {
                fallback.extend_from_slice(&[0x5A, 0x5F, 0x66, 0xFF]);
            }
            return Icon::from_rgba(fallback, TRAY_ICON_SIZE, TRAY_ICON_SIZE);
        }
    };

    Icon::from_rgba(icon.rgba.clone(), icon.width, icon.height)
}

fn load_icons() -> Result<LoadedIcons, String> {
    Ok(LoadedIcons {
        muted: decode_icon(MUTED_ICON_PNG, COLOR_MUTED)?,
        unmuted: decode_icon(UNMUTED_ICON_PNG, COLOR_UNMUTED)?,
        unknown: decode_icon(UNMUTED_ICON_PNG, COLOR_UNKNOWN)?,
    })
}

fn decode_icon(bytes: &[u8], tint: [u8; 3]) -> Result<IconImage, String> {
    let mut image = image::load_from_memory(bytes)
        .map_err(|error| error.to_string())?
        .resize(TRAY_ICON_SIZE, TRAY_ICON_SIZE, FilterType::Lanczos3)
        .to_rgba8();

    // Keep the original transparency and edge smoothing, but replace visible pixels
    // with a state-specific tint so the tray icon communicates mute status clearly.
    for pixel in image.pixels_mut() {
        if pixel[3] == 0 {
            continue;
        }

        pixel[0] = tint[0];
        pixel[1] = tint[1];
        pixel[2] = tint[2];
    }

    Ok(IconImage {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}
