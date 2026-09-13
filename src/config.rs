use serde::Deserialize;
use std::collections::BTreeSet;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub shared_ports: BTreeSet<u16>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shared_ports: BTreeSet::new(),
        }
    }
}

pub fn get_config() -> Result<Config> {
    let path = std::env::var_os("DEVPTP_CONFIG")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let local = std::path::PathBuf::from(".devptp.toml");
            local.exists().then_some(local)
        })
        .or_else(|| {
            std::env::var_os("XDG_CONFIG_HOME").map(|root| {
                std::path::PathBuf::from(root).join("devptp/config.toml")
            })
        })
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                std::path::PathBuf::from(home).join(".config/devptp/config.toml")
            })
        });

    let Some(path) = path else {
        return Ok(Config::default());
    };

    if !path.exists() {
        if std::env::var_os("DEVPTP_CONFIG").is_some() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("configuration file not found: {}", path.display()),
            )
            .into());
        }
        return Ok(Config::default());
    }

    let contents = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}
