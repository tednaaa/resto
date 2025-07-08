use reqwest::{Client, IntoUrl, Method};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::HttpMethod;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

#[derive(Clone)]
pub struct HttpClient {
	client: Client,
	cookies_store: Arc<CookieStoreMutex>,
}

impl HttpClient {
	pub fn new() -> Self {
		let cookies_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));

		let client = Client::builder()
			.timeout(Duration::from_secs(30))
			.user_agent(format!("{} HTTP Client/1.0", env!("CARGO_PKG_NAME")))
			.cookie_store(true)
			.cookie_provider(cookies_store.clone())
			.build()
			.unwrap_or_else(|_| Client::new());

		Self { client, cookies_store }
	}

	pub async fn send_request(&self, request: &HttpRequest) -> anyhow::Result<HttpResponse> {
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

		Ok(HttpResponse::new(request.id.clone(), status_code, status_text, headers, body, response_time))
	}

	pub fn add_cookies(&self, cookies: Vec<String>, url: impl IntoUrl) -> anyhow::Result<()> {
		if cookies.is_empty() {
			return Ok(());
		}

		let url = url.into_url()?;

		{
			let mut cookies_store =
				self.cookies_store.lock().map_err(|_| anyhow::anyhow!("Failed to acquire cookies store lock"))?;

			for cookie in cookies {
				let _ = cookies_store.parse(&cookie, &url);
			}
		}

		Ok(())
	}

	pub fn get_cookies(&self) -> anyhow::Result<Vec<String>> {
		let cookies = {
			let cookies_store =
				self.cookies_store.lock().map_err(|_| anyhow::anyhow!("Failed to acquire cookies store lock"))?;

			cookies_store.iter_any().map(|cookie| cookie.to_string()).collect()
		};
		Ok(cookies)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_add_cookies_empty_vector() {
		let client = HttpClient::new();
		let cookies = vec![];
		let result = client.add_cookies(cookies, "https://example.com");

		assert!(result.is_ok());

		let cookies = client.get_cookies().unwrap();
		assert!(cookies.is_empty());
	}

	#[test]
	fn test_add_cookies_single_cookie() {
		let client = HttpClient::new();
		let cookies = vec![String::from("session=abc123")];
		let result = client.add_cookies(cookies, "https://example.com");

		assert!(result.is_ok());

		let cookies = client.get_cookies().unwrap();
		assert!(cookies.contains(&String::from("session=abc123")));
	}

	#[test]
	fn test_add_cookies_multiple_cookies() {
		let client = HttpClient::new();
		let new_cookies = vec![String::from("session=abc123"), String::from("user=john"), String::from("theme=dark")];
		let result = client.add_cookies(new_cookies, "https://example.com");
		assert!(result.is_ok());

		let cookies = client.get_cookies().unwrap();
		assert!(cookies.contains(&String::from("session=abc123")));
		assert!(cookies.contains(&String::from("user=john")));
		assert!(cookies.contains(&String::from("theme=dark")));
	}

	#[test]
	fn test_add_cookies_with_attributes() {
		let client = HttpClient::new();
		let cookies =
			vec![String::from("session=abc123; Path=/; HttpOnly"), String::from("user=john; Domain=example.com; Secure")];
		let result = client.add_cookies(cookies, "https://example.com");

		assert!(result.is_ok());

		let cookies = client.get_cookies().unwrap();
		assert!(cookies.contains(&String::from("session=abc123; HttpOnly; Path=/")));
		assert!(cookies.contains(&String::from("user=john; Secure; Domain=example.com")));
	}

	#[test]
	fn test_add_cookies_invalid_url() {
		let client = HttpClient::new();
		let cookies = vec![String::from("session=abc123")];
		let result = client.add_cookies(cookies, "invalid-url");

		assert!(result.is_err());

		let cookies = client.get_cookies().unwrap();
		assert!(cookies.is_empty());
	}

	#[test]
	fn test_add_cookies_malformed_cookie() {
		let client = HttpClient::new();
		let cookies = vec![String::from("invalid cookie format")];
		let result = client.add_cookies(cookies, "https://example.com");

		assert!(result.is_ok());

		let cookies = client.get_cookies().unwrap();
		assert!(cookies.is_empty());
	}

	// TODO: also test urls
	#[test]
	fn test_add_cookies_different_domains() {
		let client = HttpClient::new();

		let cookies1 = vec![String::from("session=abc123")];
		let result1 = client.add_cookies(cookies1, "https://example.com");
		assert!(result1.is_ok());

		let cookies2 = vec![String::from("user=john")];
		let result2 = client.add_cookies(cookies2, "https://test.com");
		assert!(result2.is_ok());

		let cookies = client.get_cookies().unwrap();
		assert!(cookies.contains(&String::from("session=abc123")));
		assert!(cookies.contains(&String::from("user=john")));
	}
}
