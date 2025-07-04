use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{app::HttpMethod, utils::format_key_values::format_key_values};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
	pub id: String,
	pub method: HttpMethod,
	pub url: String,
	pub headers: HashMap<String, String>,
	pub queries: HashMap<String, String>,
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
			queries: HashMap::new(),
			body: String::new(),
			created_at: chrono::Utc::now(),
		}
	}

	pub fn set_url(&mut self, url: &str) {
		if url.starts_with("http") {
			self.url = String::from(url);
		} else {
			self.url = format!("https://{url}");
		}
	}

	pub const fn set_method(&mut self, method: HttpMethod) {
		self.method = method;
	}

	pub fn add_header(&mut self, key: String, value: String) {
		self.headers.insert(key, value);
	}

	pub fn add_query(&mut self, key: String, value: String) {
		self.queries.insert(key, value);
	}

	pub fn set_body(&mut self, body: &str) -> anyhow::Result<()> {
		let json_value: serde_json::Value = serde_json::from_str(body)?;
		self.body = serde_json::to_string_pretty(&json_value)?;
		Ok(())
	}

	pub fn is_valid(&self) -> bool {
		!self.url.is_empty() && self.url.starts_with("http")
	}

	pub fn content_type(&self) -> Option<&String> {
		self.headers.get("Content-Type").or_else(|| self.headers.get("content-type"))
	}

	pub const fn has_body(&self) -> bool {
		matches!(self.method, HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch)
	}

	pub fn formatted_headers(&self) -> String {
		format_key_values(&self.headers)
	}

	pub fn formatted_queries(&self) -> String {
		format_key_values(&self.queries)
	}
}

impl Default for HttpRequest {
	fn default() -> Self {
		Self::new()
	}
}
