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
			.user_agent(format!("{} HTTP Client/1.0", env!("CARGO_PKG_NAME")))
			.build()
			.unwrap_or_else(|_| Client::new());

		Self { client }
	}

	pub async fn send_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
		let start_time = Instant::now();

		let method = self.convert_method(&request.method);
		let mut request_builder = self.client.request(method, &request.url);

		for (key, value) in &request.headers {
			request_builder = request_builder.header(key, value);
		}

		if request.has_body() && !request.body.is_empty() {
			request_builder = request_builder.body(request.body.clone());
		}

		let response = request_builder.send().await?;
		let response_time = start_time.elapsed();

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

	const fn convert_method(&self, method: &HttpMethod) -> Method {
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
