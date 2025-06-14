use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
	pub id: String,
	pub request_id: String,
	pub status_code: u16,
	pub status_text: String,
	pub headers: HashMap<String, String>,
	pub body: String,
	pub response_time: u64, // milliseconds
	pub size: usize,        // bytes
	pub created_at: chrono::DateTime<chrono::Utc>,
}

impl HttpResponse {
	pub fn new(
		request_id: String,
		status_code: u16,
		status_text: String,
		headers: HashMap<String, String>,
		body: String,
		response_time: Duration,
	) -> Self {
		let size = body.len();

		Self {
			id: uuid::Uuid::new_v4().to_string(),
			request_id,
			status_code,
			status_text,
			headers,
			body,
			#[allow(clippy::cast_possible_truncation)]
			response_time: response_time.as_millis() as u64,
			size,
			created_at: chrono::Utc::now(),
		}
	}

	pub const fn is_success(&self) -> bool {
		self.status_code >= 200 && self.status_code < 300
	}

	pub const fn is_client_error(&self) -> bool {
		self.status_code >= 400 && self.status_code < 500
	}

	pub const fn is_server_error(&self) -> bool {
		self.status_code >= 500
	}

	pub fn content_type(&self) -> Option<&String> {
		self.headers.get("content-type").or_else(|| self.headers.get("Content-Type"))
	}

	pub fn is_json(&self) -> bool {
		self.content_type().is_some_and(|ct| ct.contains("application/json"))
	}

	pub fn is_xml(&self) -> bool {
		self.content_type().is_some_and(|ct| ct.contains("application/xml") || ct.contains("text/xml"))
	}

	pub fn is_html(&self) -> bool {
		self.content_type().is_some_and(|ct| ct.contains("text/html"))
	}

	pub fn formatted_headers(&self) -> String {
		self.headers.iter().map(|(key, value)| format!("{key}: {value}")).collect::<Vec<_>>().join("\n")
	}

	pub fn pretty_json(&self) -> anyhow::Result<String, serde_json::Error> {
		if self.is_json() {
			let json_value: serde_json::Value = serde_json::from_str(&self.body)?;
			serde_json::to_string_pretty(&json_value)
		} else {
			Ok(self.body.clone())
		}
	}

	pub fn formatted_size(&self) -> String {
		if self.size < 1024 {
			format!("{} B", self.size)
		} else if self.size < 1024 * 1024 {
			format!("{:.1} KB", self.size as f64 / 1024.0)
		} else {
			format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
		}
	}

	pub const fn status_color(&self) -> Color {
		match self.status_code {
			200..=299 => Color::Green,
			300..=399 => Color::Yellow,
			400..=499 => Color::Red,
			500..=599 => Color::Magenta,
			_ => Color::White,
		}
	}
}
