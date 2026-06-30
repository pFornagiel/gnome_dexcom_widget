use anyhow::{Context, Result};
use chrono::Utc;
use dexrs::dexcom::client::DexcomClient;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::signal;
use tokio::time::{self, Duration};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GlucoseStatus {
    mg_dl: u16,
    trend_arrow: String,
    diff: Option<i32>,
    /// Unix epoch seconds when this status was written.
    timestamp: i64,
}

// ---------------------------------------------------------------------------
// Config (read once at startup)
// ---------------------------------------------------------------------------

struct Config {
    username: String,
    password: String,
    ous: bool,
    status_file: PathBuf,
}

impl Config {
    fn from_env() -> Result<Self> {
        let username = env::var("DEXCOM_USERNAME").context("DEXCOM_USERNAME not set")?;
        let password = env::var("DEXCOM_PASSWORD").context("DEXCOM_PASSWORD not set")?;
        let ous = env::var("DEXCOM_OUS").map(|v| v == "true").unwrap_or(false);

        let dirs = directories::ProjectDirs::from("com", "pawel", "glucose-monitor")
            .context("Could not determine project dirs")?;

        let runtime_dir = dirs
            .runtime_dir()
            .or(Some(dirs.cache_dir()))
            .context("No runtime or cache dir")?
            .to_path_buf();

        std::fs::create_dir_all(&runtime_dir)?;

        Ok(Self {
            username,
            password,
            ous,
            status_file: runtime_dir.join("status.json"),
        })
    }
}

// ---------------------------------------------------------------------------
// Client helpers (all blocking — called via spawn_blocking)
// ---------------------------------------------------------------------------

fn create_client(cfg: &Config) -> Result<DexcomClient> {
    DexcomClient::new(cfg.username.clone(), cfg.password.clone(), cfg.ous)
        .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))
}

fn fetch_reading(client: &DexcomClient) -> Result<(u16, String, String)> {
    let readings = client
        .get_glucose_readings(Some(1440), Some(1))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let reading = readings
        .first()
        .ok_or_else(|| anyhow::anyhow!("No readings returned"))?;

    Ok((
        reading.mg_dl,
        reading.trend.arrow.to_string(),
        reading.datetime.clone(),
    ))
}

// heuristics, more or less
fn is_session_error(err: &anyhow::Error) -> bool {
    let msg = format!("{:?}", err).to_lowercase();
    msg.contains("session")
        || msg.contains("unauthorized")
        || msg.contains("401")
        || msg.contains("account id")
}

async fn load_previous_status(path: &PathBuf) -> Option<GlucoseStatus> {
    let data = fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&data).ok()
}

fn compute_sleep_secs(reading_datetime: &str) -> u64 {
    const DEXCOM_INTERVAL: i64 = 300; // 5 minutes
    const BUFFER: i64 = 30;
    const MIN_SLEEP: i64 = 30;
    const MAX_SLEEP: i64 = DEXCOM_INTERVAL + BUFFER;

    if let Ok(ts) = chrono::DateTime::parse_from_str(reading_datetime, "%Y-%m-%dT%H:%M:%S%z") {
        let age_secs = Utc::now().timestamp() - ts.timestamp();
        let ideal = DEXCOM_INTERVAL - age_secs + BUFFER;
        return ideal.clamp(MIN_SLEEP, MAX_SLEEP) as u64;
    }

    // Fallback if we can't parse the timestamp
    (DEXCOM_INTERVAL + BUFFER) as u64
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Arc::new(Config::from_env()?);

    eprintln!("glucose-monitor: starting for user {}", cfg.username);
    eprintln!("glucose-monitor: writing to {:?}", cfg.status_file);

    let mut last_mg_dl: Option<u16> = load_previous_status(&cfg.status_file)
        .await
        .map(|s| s.mg_dl);

    if let Some(v) = last_mg_dl {
        eprintln!(
            "glucose-monitor: restored previous value {} mg/dL from disk",
            v
        );
    }

    let mut client: Option<DexcomClient> = None;

    loop {
        if client.is_none() {
            eprintln!("glucose-monitor: (re)authenticating...");
            let cfg_ref = Arc::clone(&cfg);
            match tokio::task::spawn_blocking(move || create_client(&cfg_ref)).await {
                Ok(Ok(c)) => {
                    eprintln!("glucose-monitor: authenticated");
                    client = Some(c);
                }
                Ok(Err(e)) => {
                    eprintln!("glucose-monitor: auth failed: {e}");
                    // Back off before retrying
                    time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
                Err(e) => {
                    // dexrs panics internally (unwrap) on transient API errors;
                    // catch the panic here instead of crashing the whole process.
                    eprintln!("glucose-monitor: client creation panicked: {e}");
                    time::sleep(Duration::from_secs(60)).await;
                    continue;
                }
            }
        }

        let c = client.take().unwrap();
        let result = match tokio::task::spawn_blocking(move || {
            let res = fetch_reading(&c);
            (c, res)
        })
        .await
        {
            Ok(r) => r,
            Err(e) => {
                // dexrs may also panic during fetch; handle gracefully.
                eprintln!("glucose-monitor: fetch panicked: {e}");
                client = None;
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        let (returned_client, fetch_result) = result;

        let sleep_secs = match fetch_result {
            Ok((mg_dl, trend_arrow, reading_datetime)) => {
                let diff = last_mg_dl.map(|prev| mg_dl as i32 - prev as i32);
                last_mg_dl = Some(mg_dl);

                let status = GlucoseStatus {
                    mg_dl,
                    trend_arrow: trend_arrow.clone(),
                    diff,
                    timestamp: Utc::now().timestamp(),
                };

                let json = serde_json::to_string(&status)?;
                fs::write(&cfg.status_file, &json)
                    .await
                    .context("Failed to write status file")?;

                eprintln!(
                    "glucose-monitor: {} mg/dL {} (diff {:+?})  next in ~{}s",
                    mg_dl,
                    trend_arrow,
                    diff,
                    compute_sleep_secs(&reading_datetime),
                );

                client = Some(returned_client);

                compute_sleep_secs(&reading_datetime)
            }
            Err(e) => {
                eprintln!("glucose-monitor: fetch error: {e}");

                if is_session_error(&e) {
                    eprintln!("glucose-monitor: session expired, will re-authenticate");
                    client = None;
                } else {
                    client = Some(returned_client);
                }

                60 // retry after 1 minute on error
            }
        };

        // race the sleep against SIGTERM/Ctrl-C in case of shutdown
        tokio::select! {
            _ = time::sleep(Duration::from_secs(sleep_secs)) => {
                // Normal wake-up, loop again
            }
            _ = signal::ctrl_c() => {
                eprintln!("glucose-monitor: shutting down gracefully");
                break;
            }
        }
    }

    Ok(())
}
