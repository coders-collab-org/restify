use std::{fs::read_to_string, path::Path};

use lazy_static::lazy_static;
use serde::Deserialize;
use syn::{parse_str, Type};

lazy_static! {
  pub static ref CONFIG: Config = Config::new();
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
  pub state: Option<String>,
}

impl Config {
  pub fn new() -> Self {
    let path = Path::new("restify.toml");

    if !path.exists() {
      return Config::default();
    }

    let config: Config =
      toml::from_str(&read_to_string(path).expect("Failed to to read restify.toml"))
        .expect("Failed to parse restify.toml");

    if let Some(path) = &config.state {
      parse_str::<Type>(path).expect("Invalid state-type-path");
    }

    config
  }
}
