use anyhow::Result;
use reqwest::{Client, Method};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::app::HttpMethod;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub struct HttpClient {
	client: Client,
}

impl HttpClient {
	pub fn new() -> Self {
		let client = Client::builder()
			.timeout(Duration::from_secs(30))
			.user_agent("resto HTTP Client/1.0")
			.build()
			.unwrap_or_else(|_| Client::new());

		Self { client }
	}

	pub async fn send_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
		let start_time = Instant::now();

		let method = self.convert_method(&request.method);
		let mut req_builder = self.client.request(method, &request.url);

		// Add headers
		for (key, value) in &request.headers {
			req_builder = req_builder.header(key, value);
		}

		// Add body if applicable
		if request.has_body() && !request.body.is_empty() {
			req_builder = req_builder.body(request.body.clone());
		}

		// Send request
		let response = req_builder.send().await?;
		let response_time = start_time.elapsed();

		// Extract response data
		let status_code = response.status().as_u16();
		let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();

		let mut headers = HashMap::new();
		for (key, value) in response.headers() {
			if let Ok(value_str) = value.to_str() {
				headers.insert(key.to_string(), value_str.to_string());
			}
		}

		let body = response.text().await?;

		Ok(HttpResponse::new(
			request.id.clone(),
			status_code,
			status_text,
			headers,
			body,
			response_time,
		))
	}

	fn convert_method(&self, method: &HttpMethod) -> Method {
		match method {
			HttpMethod::Get => Method::GET,
			HttpMethod::Post => Method::POST,
			HttpMethod::Put => Method::PUT,
			HttpMethod::Delete => Method::DELETE,
			HttpMethod::Patch => Method::PATCH,
			HttpMethod::Head => Method::HEAD,
			HttpMethod::Options => Method::OPTIONS,
		}
	}
}

impl Default for HttpClient {
	fn default() -> Self {
		Self::new()
	}
}
