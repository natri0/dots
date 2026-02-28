#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["rt", "macros"] }

serde = { version = "1", features = ["derive"] }
toml = "1"
hex = { version = "0.4", features = ["serde"] }

ed25519-dalek = { version = "2.2.0", features = ["serde"] }
---

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use axum::{Router, routing::get, response::IntoResponse};
use axum::http::{status::StatusCode, header::HeaderMap};
use axum::extract::{Path, State};

use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH, Signature};

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
  listen_addr: String,

  #[serde(with = "hex")]
  pubkey: [u8; PUBLIC_KEY_LENGTH],

  base_dir: String,

  #[serde(default)]
  commands: HashMap<String, CommandConfig>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CommandConfig {
  command: String,
  workdir: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
  let config_path = std::env::args().nth(1)
    .expect("no config path given");

  let config = fs::read_to_string(config_path)
    .expect("bad config path");

  let config: Config = toml::from_str(&config)
    .expect("bad config");

  let config = Arc::new(config);

  let app = Router::new()
    .route("/run/{cmd}", get(handle_run))
    .with_state(config.clone());

  axum::serve(
    tokio::net::TcpListener::bind(&config.listen_addr).await
      .expect("could not bind"),
    app
  ).await.unwrap()
}

async fn handle_run(
  Path(cmd): Path<String>,
  State(config): State<Arc<Config>>,
  headers: HeaderMap
) -> impl IntoResponse {
  if !config.commands.contains_key(&cmd) {
    return (StatusCode::NOT_FOUND, "no such command");
  }

  let Ok(pubkey) = VerifyingKey::from_bytes(&config.pubkey) else {
    return (StatusCode::INTERNAL_SERVER_ERROR, "bad pubkey in config");
  };

  if !check_signature(&cmd, &headers, &pubkey) {
    return (StatusCode::BAD_REQUEST, "missing or invalid signature");
  }

  (StatusCode::OK, "ran successfully")
}

fn check_signature(cmd: &str, headers: &HeaderMap, pubkey: &VerifyingKey) -> bool {
  let Some(signature) = headers.get("CD-Signature") else { return false; };
  let Ok(signature) = hex::decode(signature) else { return false; };
  let Ok(signature) = Signature::from_slice(&signature) else { return false; };

  pubkey.verify_strict(cmd.as_bytes(), &signature).is_ok()
}
