use serde::{Deserialize, Serialize};

use super::{NetworkConfig, VoiceConfig};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config
{
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub voice: VoiceConfig,
}

impl Config
{
    pub fn from_file(path: &str) -> Result<Config,String>
    {
        let file = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        //read toml config
        let config: Config = toml::from_str(&file).map_err(|e| e.to_string())?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(),String>
    {
        let config = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, config).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Default for Config
{
    fn default() -> Self {
        Config{
            network: NetworkConfig::default(),
            voice: VoiceConfig::default(),
        }
    }
}

