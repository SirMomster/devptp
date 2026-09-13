use serde::Deserialize;
use std::collections::BTreeSet;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub shared_ports: BTreeSet<u16>,
}

pub fn get_config() -> Result<Config> {
    let contents = std::fs::read_to_string(".devptp.toml")?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}
