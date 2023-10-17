use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VoiceConfig
{
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    #[serde(default = "VoiceConfig::default_gain")]
    pub gain: i32,
}

impl VoiceConfig
{
    fn default_gain() -> i32 { 0 }
}

impl Default for VoiceConfig
{
    fn default() -> Self {
        Self { 
            input_device: None,
            output_device: None,
            gain: VoiceConfig::default_gain(),
        }
    }
}