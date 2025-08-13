use clap::{Arg, Command};
use std::process;
use tracing::{error, info, Level};
use tracing_subscriber;

mod config;
mod proxy;
mod script_manager;
mod http_injector;

use config::Config;
use proxy::ProxyServer;
use script_manager::ScriptManager;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let matches = Command::new("rusty-proxy")
        .version("0.1.0")
        .about("HTTP Proxy Script Manager for Traffic Injection")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("config.toml"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Proxy server port")
                .default_value("8080"),
        )
        .arg(
            Arg::new("scripts-dir")
                .short('s')
                .long("scripts-dir")
                .value_name("DIR")
                .help("Directory containing injection scripts")
                .default_value("scripts"),
        )
        .subcommand(
            Command::new("start")
                .about("Start the proxy server")
        )
        .subcommand(
            Command::new("list-scripts")
                .about("List available injection scripts")
        )
        .subcommand(
            Command::new("install")
                .about("Install as system service")
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let port: u16 = matches.get_one::<String>("port").unwrap().parse().unwrap_or(8080);
    let scripts_dir = matches.get_one::<String>("scripts-dir").unwrap();

    // Load configuration
    let config = match Config::load(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load config: {}", e);
            process::exit(1);
        }
    };

    // Initialize script manager
    let script_manager = match ScriptManager::new(scripts_dir) {
        Ok(sm) => sm,
        Err(e) => {
            error!("Failed to initialize script manager: {}", e);
            process::exit(1);
        }
    };

    match matches.subcommand() {
        Some(("start", _)) => {
            info!("Starting proxy server on port {}", port);
            let proxy = ProxyServer::new(port, config, script_manager);
            if let Err(e) = proxy.run().await {
                error!("Proxy server error: {}", e);
                process::exit(1);
            }
        }
        Some(("list-scripts", _)) => {
            let scripts = script_manager.list_scripts();
            println!("Available injection scripts:");
            for script in scripts {
                println!("  - {}", script);
            }
        }
        Some(("install", _)) => {
            if let Err(e) = install_service() {
                error!("Failed to install service: {}", e);
                process::exit(1);
            }
            info!("Service installed successfully");
        }
        _ => {
            info!("Starting proxy server on port {} (default)", port);
            let proxy = ProxyServer::new(port, config, script_manager);
            if let Err(e) = proxy.run().await {
                error!("Proxy server error: {}", e);
                process::exit(1);
            }
        }
    }
}

fn install_service() -> anyhow::Result<()> {
    use std::fs;
    use std::path::Path;

    let service_content = r#"[Unit]
Description=Rusty Proxy HTTP Injector
After=network.target

[Service]
Type=simple
User=rusty-proxy
WorkingDirectory=/opt/rusty-proxy
ExecStart=/opt/rusty-proxy/rusty-proxy start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"#;

    let service_path = "/etc/systemd/system/rusty-proxy.service";
    if Path::new(service_path).exists() {
        println!("Service already exists at {}", service_path);
        return Ok(());
    }

    fs::write(service_path, service_content)?;
    
    // Enable and start the service
    std::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status()?;
    
    std::process::Command::new("systemctl")
        .args(["enable", "rusty-proxy"])
        .status()?;

    println!("Systemd service installed. Start with: sudo systemctl start rusty-proxy");
    Ok(())
}