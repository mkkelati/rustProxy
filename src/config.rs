use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub proxy: ProxyConfig,
    pub scripts: ScriptConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    pub bind_address: String,
    pub port: u16,
    pub upstream_timeout: u64,
    pub max_connections: usize,
    pub buffer_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptConfig {
    pub directory: String,
    pub enabled: bool,
    pub max_execution_time: u64,
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub max_size: String,
    pub max_files: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub require_auth: bool,
    pub auth_token: Option<String>,
    pub rate_limit: u32,
    pub whitelist_ips: Vec<String>,
    pub blacklist_ips: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            proxy: ProxyConfig {
                bind_address: "127.0.0.1".to_string(),
                port: 8080,
                upstream_timeout: 30,
                max_connections: 1000,
                buffer_size: 8192,
            },
            scripts: ScriptConfig {
                directory: "scripts".to_string(),
                enabled: true,
                max_execution_time: 5000,
                allowed_domains: vec!["*".to_string()],
                blocked_domains: vec![],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: Some("rusty-proxy.log".to_string()),
                max_size: "10MB".to_string(),
                max_files: 5,
            },
            security: SecurityConfig {
                require_auth: false,
                auth_token: None,
                rate_limit: 100,
                whitelist_ips: vec![],
                blacklist_ips: vec![],
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            let default_config = Config::default();
            default_config.save(&path)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        if self.scripts.blocked_domains.contains(&domain.to_string()) {
            return false;
        }

        if self.scripts.allowed_domains.contains(&"*".to_string()) {
            return true;
        }

        self.scripts.allowed_domains.iter().any(|allowed| {
            domain == allowed || domain.ends_with(&format!(".{}", allowed))
        })
    }

    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        if !self.security.blacklist_ips.is_empty() && self.security.blacklist_ips.contains(&ip.to_string()) {
            return false;
        }

        if self.security.whitelist_ips.is_empty() {
            return true;
        }

        self.security.whitelist_ips.contains(&ip.to_string())
    }
}