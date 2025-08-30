use std::path::PathBuf;

use lazy_static::lazy_static;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, Layer, layer::SubscriberExt, util::SubscriberInitExt};

lazy_static! {
	pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
	pub static ref LOG_ENV: String = format!("{}_LOGLEVEL", PROJECT_NAME.clone());
	pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

pub fn get_data_dir() -> PathBuf {
	PathBuf::from(".").join(".data")
}

pub fn initialize_logging() -> anyhow::Result<()> {
	let directory = get_data_dir();
	std::fs::create_dir_all(directory.clone())?;
	let log_path = directory.join(LOG_FILE.clone());
	let log_file = std::fs::File::create(log_path)?;

	let file_subscriber = tracing_subscriber::fmt::layer()
		.with_file(true)
		.with_line_number(true)
		.with_writer(log_file)
		.with_target(false)
		.with_ansi(false)
		.with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
	tracing_subscriber::registry().with(file_subscriber).with(ErrorLayer::default()).init();
	Ok(())
}

#[macro_export]
macro_rules! trace_dbg {
	(target: $target:expr, level: $level:expr, $ex:expr) => {{
		match $ex {
			value => {
				tracing::event!(target: $target, $level, ?value, stringify!($ex));
				value
			}
		}
	}};
	(level: $level:expr, $ex:expr) => {
		trace_dbg!(target: module_path!(), level: $level, $ex)
	};
	(target: $target:expr, $ex:expr) => {
		trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
	};
	($ex:expr) => {
		trace_dbg!(level: tracing::Level::DEBUG, $ex)
	};
}
