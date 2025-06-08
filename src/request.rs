use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::app::HttpMethod;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HttpRequest {
	pub id: String,
	pub method: HttpMethod,
	pub url: String,
	pub headers: HashMap<String, String>,
	pub body: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
}

impl HttpRequest {
	pub fn new() -> Self {
		Self {
			id: Uuid::new_v4().to_string(),
			method: HttpMethod::Get,
			url: String::new(),
			headers: HashMap::new(),
			body: String::new(),
			created_at: chrono::Utc::now(),
		}
	}

	pub fn with_url(mut self, url: String) -> Self {
		self.url = url;
		self
	}

	pub const fn with_method(mut self, method: HttpMethod) -> Self {
		self.method = method;
		self
	}

	pub fn with_header(mut self, key: String, value: String) -> Self {
		self.headers.insert(key, value);
		self
	}

	pub fn with_body(mut self, body: String) -> Self {
		self.body = body;
		self
	}

	pub fn is_valid(&self) -> bool {
		!self.url.is_empty() && self.url.starts_with("http")
	}

	pub fn content_type(&self) -> Option<&String> {
		self.headers
			.get("Content-Type")
			.or_else(|| self.headers.get("content-type"))
	}

	pub const fn has_body(&self) -> bool {
		matches!(self.method, HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch)
	}

	pub fn formatted_headers(&self) -> String {
		self.headers
			.iter()
			.map(|(key, value)| format!("{key}: {value}"))
			.collect::<Vec<_>>()
			.join("\n")
	}
}

impl Default for HttpRequest {
	fn default() -> Self {
		Self::new()
	}
}
