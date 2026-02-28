#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["rt", "macros"] }

serde = { version = "1", features = ["derive"] }
toml = "1"
hex = { version = "0.4", features = ["serde"] }
---

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use axum::{Router, routing::get, response::IntoResponse, http::status::StatusCode};
use axum::extract::{Path, State};

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
  listen_addr: String,

  #[serde(with = "hex")]
  pubkey: [u8; 32],

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
  State(config): State<Arc<Config>>
) -> impl IntoResponse {
  if !config.commands.contains_key(&cmd) {
    return (StatusCode::NOT_FOUND, "no such command");
  }

  (StatusCode::OK, "ran successfully")
}