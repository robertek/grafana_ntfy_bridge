use serde::Deserialize;
use std::{
	path::Path,
	error, fs
};

use crate::Cli;

const DEFAULT_CONF: &str = "grafana_to_ntfy.toml";
const DEFAULT_URL: &str = "http://ntfy.sh";
const DEFAULT_PORT: u16 = 8080;

#[derive(Deserialize, Debug)]
struct ConfigFile {
	url: Option<String>,
	topic: Option<String>,
	port: Option<u16>,
	key: Option<String>,
}

impl ConfigFile {
	fn new(path: String) -> Result<Self, Box<dyn error::Error>> {
		let file = Path::new(&path);
		let config_file: ConfigFile;

		if file.exists() {
			config_file = toml::from_str(fs::read_to_string(&file)?.as_str())?;
		} else {
			return Err(path.into());
		}

		Ok(config_file)
	}
}

//#[derive(Copy)]
pub struct Config {
	pub url: String,
	pub topic: String,
	pub port: u16,
	pub key: Option<String>,
}

impl Config {
	pub fn new(args: Cli) -> Self {
		tracing::debug!("{:#?}", args);
		let config_path = match args.config_file {
			Some(c) => c,
			None => DEFAULT_CONF.to_string(),
		};

		let config_file = match ConfigFile::new(config_path) {
			Ok(c) => c,
			Err(e) => {
				tracing::warn!(e);
				ConfigFile {
					url: Some(DEFAULT_URL.to_string()),
					topic: None, 
					port: Some(DEFAULT_PORT), 
					key: None}
			}
		};

		let mut config: Config = Config {
			url: DEFAULT_URL.to_string(),
			topic: "".to_string(), 
			port: DEFAULT_PORT, 
			key: None}; 

		config.port = match args.port {
			Some(p) => p,
			None => match config_file.port {
				Some(p) => p,
				None => DEFAULT_PORT
			}
		};
		tracing::debug!("config port: {}", config.port);

		config.key = match args.key {
			Some(k) => Some(k),
			None => config_file.key
		};
		tracing::debug!("config key: {:?}", config.key);

		match args.topic {
			Some(t) => config.topic = t,
			None => match config_file.topic {
				Some(t) => config.topic = t,
				None => {
					tracing::error!("Missing ntfy topic");
					std::process::exit(exitcode::USAGE);
				}
			}
		};
		tracing::debug!("config NTFY topic: {}", config.topic);

		config.url = match args.url {
			Some(u) => u,
			None => match config_file.url {
				Some(u) => u,
				None => DEFAULT_URL.to_string()
			}
		};
		tracing::debug!("config url: {}", config.url);

		config
	}
}

