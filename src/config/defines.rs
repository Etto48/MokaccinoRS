use std::net::IpAddr;

use eframe::epaint::Color32;

pub const MIN_LOAD_TIME: std::time::Duration = std::time::Duration::from_secs(2);

pub const HOST: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0));
pub const THREAD_QUEUE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);
pub const THREAD_SUPERVISOR_SLEEP_TIME: std::time::Duration = std::time::Duration::from_millis(200);
pub const MAX_THREAD_JOIN_TRIES: u32 = 10;
pub const MAX_PACKET_SIZE: usize = 1024;
pub const MAX_FIND_TTL: u8 = 10;
pub const VOICE_ENDED_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);
/// Must be one of 120, 240, 480, 960, 1920, and 2880. For 120 and 240 the encoder can't use LPC or hybrid modes.
pub const VOICE_BUFFER_SIZE: usize = 1920;
pub const VOICE_TRANSMISSION_SAMPLE_RATE: usize = 48000;
pub const VOICE_TRANSMISSION_BITRATE: opus::Bitrate = opus::Bitrate::Max;
pub const VOICE_MAX_TRANSMISSION_SIZE: usize = 512;
pub const UPDATE_UI_INTERVAL: std::time::Duration = std::time::Duration::from_millis(120);
pub const MIN_GAIN: i32 = -32768;
pub const MAX_GAIN: i32 = 32767;

pub const ASYMMETRIC_KEY_GENERATOR: fn() -> Result<openssl::pkey::PKey<openssl::pkey::Private>, openssl::error::ErrorStack> = || openssl::pkey::PKey::ec_gen("secp521r1");
pub const MESSAGE_DIGEST: fn() -> openssl::hash::MessageDigest = openssl::hash::MessageDigest::sha3_512;
pub const KEY_DERIVATION_MD: fn() -> openssl::hash::MessageDigest = openssl::hash::MessageDigest::sha3_512;
pub const SYMMETRIC_ALGORITHM: fn() -> openssl::symm::Cipher = openssl::symm::Cipher::aes_256_gcm;
pub const SYMMETRIC_ALGORITHM_KEY_LEN: usize = 32;
pub const SYMMETRIC_ALGORITHM_IV_LEN: usize = 12;
pub const SYMMETRIC_ALGORITHM_TAG_LEN: usize = 16;

pub const CONFIG_PATH: &str = "config.toml";

pub const LOG_COMMAND_COLOR: Color32 = Color32::from_rgb(60, 255, 60);
pub const LOG_ERROR_COLOR: Color32 = Color32::from_rgb(255, 60, 60);

pub const ACCENT_COLOR_DARK: Color32 = Color32::from_rgb(116, 77, 169);
pub const ACCENT_COLOR_LIGHT: Color32 = Color32::from_rgb(146, 89, 209);

pub const FRAME_COLOR_DARK: Color32 = Color32::from_rgb(10,10, 10);
pub const FRAME_COLOR_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);

pub const TEXT_COLOR_DARK: Color32 = Color32::from_rgb(230, 230, 230);
pub const TEXT_COLOR_LIGHT: Color32 = Color32::from_rgb(30, 30, 30);