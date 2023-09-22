use eframe::epaint::Color32;

pub const THREAD_QUEUE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);
pub const MAX_THREAD_JOIN_TRIES: u32 = 10;

pub const LOG_EVENT_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
pub const LOG_COMMAND_COLOR: Color32 = Color32::from_rgb(60, 255, 60);
pub const LOG_ERROR_COLOR: Color32 = Color32::from_rgb(255, 60, 60);