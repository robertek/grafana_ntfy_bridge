use clap::Parser;
use axum::{
    extract::{Json, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    routing::post, Router
};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;

mod config;
use crate::config::Config;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	/// Config file
	#[arg(long,short)]
	config_file: Option<String>,

	/// NTFY url
	#[arg(long,short)]
	url: Option<String>,

	/// NTFY topic
	#[arg(long,short)]
	topic: Option<String>,

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

	let config = Config::new(Cli::parse());
	let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
	let shared_config = Arc::new(config);

	let app = Router::new()
		.route("/", post(root))
		.with_state(shared_config);

	tracing::info!("listening on {}", addr);
	let listener = TcpListener::bind(&addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn root(
	headers: HeaderMap,
	State(config): State<Arc<Config>>,
	Json(payload): Json<Value>
) -> StatusCode {
	// Authorization check if the key is defined
	// Expected a standard format "Bearer key"
	match &config.key {
		Some(key) => {
			if headers.contains_key(AUTHORIZATION) {
				let configured = format!("Bearer {}", key);
				let received = headers.get(AUTHORIZATION).unwrap();
				if ! configured.eq(received) {
					return StatusCode::UNAUTHORIZED;
				}
			} else {
				return StatusCode::UNAUTHORIZED;
			}
		},
		None => {}
	}

	let tag = match payload["status"].as_str() {
		Some("firing") => "warning",
		Some("ok") => "white_check_mark",
		_ => ""
	};

	let msg = json!({
		"topic": config.topic,
		"title": payload["title"],
		"message": payload["message"],
		"tags": [ tag ],
		"click": payload["externalURL"]
	});
	tracing::debug!("{:#?}", msg);

	match send_to_ntfy(&config, msg.to_string()).await {
		Ok(_) => { return StatusCode::OK }
		Err(_) => { return StatusCode::BAD_REQUEST }
	}
}

async fn send_to_ntfy(
	config: &Config,
	msg: String
) -> Result<(), reqwest::Error> {
	tracing::debug!("Sending to {}: {}", config.url, msg);
	let client = reqwest::Client::new();
	let _res = client.post(config.url.to_string())
		.body(msg)
		.send()
		.await?;
	Ok(())
}
