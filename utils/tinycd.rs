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

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
  listen_addr: String,
  port: u16,

  #[serde(with = "hex")]
  pubkey: [u8; 32],

  base_dir: String,

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
}
