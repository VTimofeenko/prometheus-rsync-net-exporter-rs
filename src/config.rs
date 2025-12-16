use std::env;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub username: String,
    pub host: String,
    pub ssh_key_path: String,
    pub port: u16,
    pub fetch_interval_seconds: u64,
    pub listen_address: IpAddr,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let username = if let Ok(path) = env::var("RSYNC_USERNAME_FILE") {
            std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read RSYNC_USERNAME_FILE at {}: {}", path, e))?
                .trim()
                .to_string()
        } else {
            env::var("RSYNC_USERNAME")
                .map_err(|_| "RSYNC_USERNAME or RSYNC_USERNAME_FILE must be set".to_string())?
        };

        let ssh_key_path = env::var("RSYNC_SSH_KEY_PATH")
            .map_err(|_| "RSYNC_SSH_KEY_PATH must be set".to_string())?;

        let host = env::var("RSYNC_HOST").unwrap_or_else(|_| format!("{}.rsync.net", username));

        let port = env::var("RSYNC_EXPORTER_PORT")
            .unwrap_or_else(|_| "9000".to_string())
            .parse::<u16>()
            .map_err(|_| "RSYNC_EXPORTER_PORT must be a valid u16".to_string())?;

        let fetch_interval_seconds = env::var("RSYNC_FETCH_INTERVAL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .map_err(|_| "RSYNC_FETCH_INTERVAL_SECONDS must be a valid u64".to_string())?;

        let listen_address = env::var("RSYNC_LISTEN_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0".to_string())
            .parse::<IpAddr>()
            .map_err(|_| "RSYNC_LISTEN_ADDRESS must be a valid IP address".to_string())?;

        Ok(Config {
            username,
            host,
            ssh_key_path,
            port,
            fetch_interval_seconds,
            listen_address,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_success() {
        temp_env::with_vars(
            vec![
                ("RSYNC_USERNAME", Some("user123")),
                ("RSYNC_SSH_KEY_PATH", Some("/path/to/key")),
                ("RSYNC_HOST", None),
                ("RSYNC_EXPORTER_PORT", None),
                ("RSYNC_FETCH_INTERVAL_SECONDS", None),
                ("RSYNC_LISTEN_ADDRESS", None),
            ],
            || {
                let config = Config::load().expect("Should load config");
                assert_eq!(config.username, "user123");
                assert_eq!(config.ssh_key_path, "/path/to/key");
                assert_eq!(config.host, "user123.rsync.net");
                assert_eq!(config.port, 9000);
                assert_eq!(config.fetch_interval_seconds, 3600);
                assert_eq!(config.listen_address, "0.0.0.0".parse::<IpAddr>().unwrap());
            },
        );
    }

    #[test]
    fn test_config_load_custom_values() {
        temp_env::with_vars(
            vec![
                ("RSYNC_USERNAME", Some("user123")),
                ("RSYNC_SSH_KEY_PATH", Some("/path/to/key")),
                ("RSYNC_HOST", Some("custom.host.com")),
                ("RSYNC_EXPORTER_PORT", Some("8080")),
                ("RSYNC_FETCH_INTERVAL_SECONDS", Some("120")),
                ("RSYNC_LISTEN_ADDRESS", Some("127.0.0.1")),
            ],
            || {
                let config = Config::load().expect("Should load config");
                assert_eq!(config.host, "custom.host.com");
                assert_eq!(config.port, 8080);
                assert_eq!(config.fetch_interval_seconds, 120);
                assert_eq!(
                    config.listen_address,
                    "127.0.0.1".parse::<IpAddr>().unwrap()
                );
            },
        );
    }

    #[test]
    fn test_config_missing_username() {
        temp_env::with_vars(
            vec![
                ("RSYNC_USERNAME", None),
                ("RSYNC_SSH_KEY_PATH", Some("/path/to/key")),
            ],
            || {
                let err = Config::load().unwrap_err();
                assert_eq!(err, "RSYNC_USERNAME or RSYNC_USERNAME_FILE must be set");
            },
        );
    }
}
