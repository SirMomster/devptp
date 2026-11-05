use serde::Deserialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::error::Result;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub shared_ports: BTreeSet<u16>,
    pub relay_url: Option<String>,
}

pub fn get_config() -> Result<Config> {
    get_config_from(None)
}

pub fn get_config_from(config_file: Option<&Path>) -> Result<Config> {
    let explicit_path = config_file.is_some() || std::env::var_os("DEVPTP_CONFIG").is_some();
    let path = config_file
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("DEVPTP_CONFIG").map(PathBuf::from))
        .or_else(|| {
            let local = std::path::PathBuf::from(".devptp.toml");
            local.exists().then_some(local)
        })
        .or_else(|| {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(|root| std::path::PathBuf::from(root).join("devptp/config.toml"))
        })
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".config/devptp/config.toml"))
        });

    let Some(path) = path else {
        return Ok(Config::default());
    };

    if !path.exists() {
        if explicit_path {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("configuration file not found: {}", path.display()),
            )
            .into());
        }
        return Ok(Config::default());
    }

    let contents = std::fs::read_to_string(&path).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!(
                "failed to read configuration file {}: {error}",
                path.display()
            ),
        )
    })?;
    toml::from_str(&contents).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid configuration file {}: {error}", path.display()),
        )
        .into()
    })
}

#[cfg(test)]
mod tests {
    use super::get_config_from;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

    fn temp_config_path() -> PathBuf {
        let unique = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "devptp-config-{}-{unique}.toml",
            std::process::id()
        ))
    }

    #[test]
    fn loads_relay_url_from_explicit_config_file() {
        let path = temp_config_path();
        fs::write(
            &path,
            r#"shared_ports = [8083]
relay_url = "https://relay.example.com"
"#,
        )
        .expect("write config");

        let config = get_config_from(Some(&path)).expect("load config");
        assert_eq!(config.shared_ports.into_iter().collect::<Vec<_>>(), [8083]);
        assert_eq!(
            config.relay_url.as_deref(),
            Some("https://relay.example.com")
        );
        fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn relay_only_config_defaults_other_fields() {
        let config: super::Config = toml::from_str(r#"relay_url = "https://relay.example.com""#)
            .expect("relay-only config should parse");
        assert!(config.shared_ports.is_empty());
        assert_eq!(
            config.relay_url.as_deref(),
            Some("https://relay.example.com")
        );
    }

    #[test]
    fn malformed_config_is_reported_with_file_path() {
        let path = temp_config_path();
        fs::write(&path, "relay_url = [").expect("write malformed config");

        let error = get_config_from(Some(&path)).expect_err("malformed config should fail");
        assert!(error.to_string().contains("invalid configuration file"));
        assert!(error.to_string().contains(&path.display().to_string()));
        fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn missing_explicit_config_file_is_an_error() {
        let path = temp_config_path();
        let error = get_config_from(Some(&path)).expect_err("missing config should fail");
        assert!(error.to_string().contains("configuration file not found"));
    }
}
