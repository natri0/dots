#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["rt", "macros", "process"] }

serde = { version = "1", features = ["derive"] }
toml = "1"
hex = { version = "0.4", features = ["serde"] }

ed25519-dalek = { version = "2.2.0", features = ["hazmat"] }
jiff = { version = "0.2", features = ["serde"] }
---

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use axum::{Router, routing::get, response::IntoResponse};
use axum::http::{status::StatusCode, header::HeaderMap};
use axum::extract::{Path, State};

use tokio::process::Command;

use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH, Signature};
use jiff::{Zoned, Span, ToSpan, Unit, Timestamp};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Config {
  listen_addr: String,

  #[serde(with = "hex")]
  pubkey: [u8; PUBLIC_KEY_LENGTH],

  #[serde(default = "default_sign_window")] // 5m by default
  sign_window: Span,

  #[serde(default)]
  commands: HashMap<String, CommandConfig>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct CommandConfig {
  command: String,
  workdir: Option<String>,
}

/// default `sign_window` for [`Config`]: 5 minutes
fn default_sign_window() -> Span {
  5.minutes()
}

struct AppState {
  config: Config,
  config_dir: std::path::PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
  let config_path = std::env::args().nth(1)
    .expect("no config path given");

  let config = fs::read_to_string(&config_path)
    .expect("bad config path");

  let config: Config = toml::from_str(&config)
    .expect("bad config");

  println!("starting with config: {config:?}");

  let state = Arc::new(AppState {
    config,
    config_dir: fs::canonicalize(config_path)
      .expect("couldn't get config absolute path")
      .parent().unwrap().to_path_buf()
  });

  let app = Router::new()
    .route("/run/{cmd}", get(handle_run))
    .with_state(state.clone());

  axum::serve(
    tokio::net::TcpListener::bind(&state.config.listen_addr).await
      .expect("could not bind"),
    app
  ).await.unwrap()
}

async fn handle_run(
  Path(cmd): Path<String>,
  State(state): State<Arc<AppState>>,
  headers: HeaderMap
) -> impl IntoResponse {
  let config = &state.config;

  let Some(command) = config.commands.get(&cmd) else {
    return (StatusCode::NOT_FOUND, "no such command");
  };

  let Ok(pubkey) = VerifyingKey::from_bytes(&config.pubkey) else {
    return (StatusCode::INTERNAL_SERVER_ERROR, "bad pubkey in config");
  };

  if !check_signature(&cmd, &headers, &pubkey, &config.sign_window) {
    return (StatusCode::BAD_REQUEST, "missing or invalid signature or timestamp");
  }

  let Ok(mut pull) = Command::new("git")
    .arg("pull")
    .current_dir(&state.config_dir)
    .spawn() else { return (StatusCode::INTERNAL_SERVER_ERROR, "failed to spawn git pull"); };
  match pull.wait().await {
    Ok(st) if st.success() => {}
    _ => { return (StatusCode::INTERNAL_SERVER_ERROR, "failed to git pull"); }
  };

  let Ok(mut sh) = Command::new("sh")
    .arg("-c")
    .arg(&command.command)
    .current_dir(&state.config_dir.join(&command.workdir.as_deref().unwrap_or(".")))
    .spawn() else { return (StatusCode::INTERNAL_SERVER_ERROR, "failed to spawn command"); };
  match sh.wait().await {
    Ok(st) if st.success() => {}
    _ => { return (StatusCode::INTERNAL_SERVER_ERROR, "failed to run command"); }
  };

  (StatusCode::OK, "ran successfully")
}

fn check_signature(cmd: &str, headers: &HeaderMap, pubkey: &VerifyingKey, sign_window: &Span) -> bool {
  let Some(timestamp_str) = headers.get("CD-Timestamp").map(|h| h.to_str().ok()).flatten() else { return false; };
  let Ok(timestamp) = timestamp_str.parse::<Timestamp>() else { return false; };

  let Ok(time_diff) = (Timestamp::now() - timestamp).total(Unit::Second) else { return false; };
  if time_diff < 0. || !matches!(sign_window.total(Unit::Second), Ok(t) if time_diff <= t) { return false; };

  let Some(signature) = headers.get("CD-Signature") else { return false; };
  let Ok(signature) = hex::decode(signature) else { return false; };
  let Ok(signature) = Signature::from_slice(&signature) else { return false; };

  let Ok(mut verifier) = pubkey.verify_stream(&signature) else { return false; };

  verifier.update(cmd);
  verifier.update(":");
  verifier.update(timestamp_str);

  verifier.finalize_and_verify().is_ok()
}
