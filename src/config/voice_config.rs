use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VoiceConfig
{
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub gain: i32,
}

impl Default for VoiceConfig
{
    fn default() -> Self {
        Self { 
            input_device: None,
            output_device: None,
            gain: 0,
        }
    }
}