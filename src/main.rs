use clap::Parser;
use axum::{
    routing::post,
    http::StatusCode,
    Router
};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use serde::Deserialize;
use std::{
	net::SocketAddr,
	path::Path,
	error, fs
};

const DEFAULT_CONF: &str = "grafana_to_ntfy.toml";
const DEFAULT_PORT: u16 = 8080;

#[derive(Deserialize, Debug)]
struct ConfigFile {
	url: Option<String>,
	port: Option<u16>,
	key: Option<String>,
}

impl ConfigFile {
	pub fn new(path: String) -> Result<Self, Box<dyn error::Error>> {
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

struct Config {
	url: String,
	port: u16,
	key: Option<String>,
}

impl Config {
	pub fn new(args: Args) -> Self {
		let config_path = match args.config_file {
			Some(c) => c,
			None => DEFAULT_CONF.to_string(),
		};

		let config_file = match ConfigFile::new(config_path) {
			Ok(c) => c,
			Err(e) => {
				tracing::warn!(e);
				ConfigFile {url: None, port: Some(DEFAULT_PORT), key: None}
			}
		};

		let mut config: Config = Config {url: "".to_string(), port: DEFAULT_PORT, key: None}; 

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

		match args.url {
			Some(u) => config.url = u,
			None => match config_file.url {
				Some(u) => config.url = u,
				None => {
					tracing::error!("Missing ntfy URL");
					std::process::exit(exitcode::USAGE);
				}
			}
		};
		tracing::debug!("config NTFY url: {}", config.url);

		config
	}
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// Config file
	#[arg(long,short)]
	config_file: Option<String>,

	/// Full NTFY url
	#[arg(long,short)]
	url: Option<String>,

	/// port to listen on
	#[arg(long,short)]
	port: Option<u16>,

	/// grafana connector key
	#[arg(long,short)]
	key: Option<String>,
}

#[tokio::main]
async fn main() {
	tracing_subscriber::registry()
		.with(fmt::layer())
		.with(EnvFilter::from_default_env())
		.init();

	let args = Args::parse();
	tracing::debug!("{:#?}", args);

	let config = Config::new(args);

	let app = Router::new()
		.route("/", post(root));

	let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
	tracing::info!("listening on {}", addr);
	let listener = TcpListener::bind(&addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn root() -> StatusCode {
	StatusCode::OK
}
