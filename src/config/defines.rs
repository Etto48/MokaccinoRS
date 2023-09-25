use eframe::epaint::Color32;

pub const THREAD_QUEUE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);
pub const MAX_THREAD_JOIN_TRIES: u32 = 10;
pub const VOICE_PACKET_SIZE: usize = 128;

pub const LOG_COMMAND_COLOR: Color32 = Color32::from_rgb(60, 255, 60);
pub const LOG_ERROR_COLOR: Color32 = Color32::from_rgb(255, 60, 60);

pub const ACCENT_COLOR_DARK: Color32 = Color32::from_rgb(116, 77, 169);
pub const ACCENT_COLOR_LIGHT: Color32 = Color32::from_rgb(146, 89, 209);

pub const FRAME_COLOR_DARK: Color32 = Color32::from_rgb(10,10, 10);
pub const FRAME_COLOR_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);

pub const TEXT_COLOR_DARK: Color32 = Color32::from_rgb(230, 230, 230);
pub const TEXT_COLOR_LIGHT: Color32 = Color32::from_rgb(30, 30, 30);