//! easy-log-forwarding/src/main.rs

use better_logger::{LoggerSettings, RelaySettings, NetworkFormat, logger, relay};
use std::process::exit;
use std::fs::read_to_string;
use std::error::Error;
use std::sync::Arc;
use serde::Deserialize;
use chrono::Local;
use tokio::sync::Notify;
use tokio::time::{interval, Duration, MissedTickBehavior};


#[derive(Deserialize)]
#[serde(tag = "type")]
enum ConfigNetworkFormat {
    PlainText,
    JsonText { field: String },
}

impl From<ConfigNetworkFormat> for NetworkFormat {
    fn from(v: ConfigNetworkFormat) -> Self {
        match v {
            ConfigNetworkFormat::PlainText => NetworkFormat::PlainText,
            ConfigNetworkFormat::JsonText { field } => NetworkFormat::JsonText { field },
        }
    }
}

#[derive(Deserialize)]
struct Config {
    terminal_logs: bool,
    terminal_log_lvl: String,
    wasm_logging: bool,
    file_logs: bool,
    file_log_lvl: String,
    log_file_path: String,
    network_logs: bool,
    network_log_lvl: String,
    network_endpoint_url: String,
    network_format: ConfigNetworkFormat,
    debug_extra: bool,
    async_logging: bool,
    machine_name: String,
    container_name: String,
    relay_listen_address: String,
    relay_output_url: String,
    relay_output_format: ConfigNetworkFormat,
    relay_cors_allowed_origins: Vec<String>,
    relay_actix_workers: usize,
}

fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let raw = read_to_string(path)?;
    return Ok(toml::from_str(&raw)?);
}

#[tokio::main]
async fn main() {
    let now = Local::now();
    let timestamp = format!("{}", now.format("%Y-%m-%d %H:%M:%S"));

    let config_path = "config.toml";
    let config = match load_config(&config_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}: {:?}", timestamp, err);
            exit(1);
        }
    };

    let machine_name_1 = config.machine_name.clone();
    let container_name_1 = config.container_name.clone();
    let machine_name_2 = config.machine_name.clone();
    let container_name_2 = config.container_name.clone();

    let logger_settings = LoggerSettings {
        terminal_logs: config.terminal_logs,
        terminal_log_lvl: config.terminal_log_lvl,
        wasm_logging: config.wasm_logging,
        file_logs: config.file_logs,
        file_log_lvl: config.file_log_lvl,
        log_file_path: config.log_file_path,
        network_logs: config.network_logs,
        network_log_lvl: config.network_log_lvl,
        network_endpoint_url: config.network_endpoint_url,
        network_format: config.network_format.into(),
        debug_extra: config.debug_extra,
        async_logging: config.async_logging,
    };

    if let Err(err) = logger::init(logger_settings) {
        eprintln!("{}: {:?}", timestamp, err);
        exit(1);
    }

    let relay_settings = RelaySettings {
        listen_address: config.relay_listen_address,
        output_format: config.relay_output_format.into(),
        output_url: config.relay_output_url,
        cors_allowed_origins: config.relay_cors_allowed_origins,
        actix_workers: config.relay_actix_workers,
    };    

    let shutdown = Arc::new(Notify::new());
    let shutdown_for_task = shutdown.clone();
    tokio::spawn(async move {
        let mut time = interval(Duration::from_secs(12 * 60 * 60));
        time.set_missed_tick_behavior(MissedTickBehavior::Delay);
        time.tick().await;
        loop {
            tokio::select! {
                _ = time.tick() => {
                    let heartbeat = format!("\n{} - {}: {}", 
                        machine_name_2, 
                        container_name_2,
                        "HEARTBEAT - not a real error - HEARTBEAT"
                    );   
                    logger::error!("{}", heartbeat);
                }
                _ = shutdown_for_task.notified() => break,
            }
        }
    });

    match relay::start(relay_settings).await {
        Ok(_) => {
            shutdown.notify_waiters();
            let log_message_1 = format!("\n{} - {}: {}\n{}", 
                machine_name_1, 
                container_name_1,
                "EXITED WITH CONDITION: \"Ok()\"",
                "If this was not planned, is an error"
            );
            logger::warn!("{}", log_message_1);
        }
        Err(error) => {
            shutdown.notify_waiters();
            let log_message_2 = format!("\n{} - {}: {}\n{}\n{}", 
                machine_name_1, 
                container_name_1,
                "EXITED WITH CONDITION: \"Err()\"",
                "ERROR:",
                error
            );
            logger::error!("{}", log_message_2);
        }
    }
}
