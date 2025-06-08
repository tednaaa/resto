use anyhow::Result;

use crate::request::HttpRequest;

pub fn parse_curl(input: &str) -> Result<HttpRequest> {
	let request = HttpRequest::new();

	let mut parts = input.split_whitespace();

	if let Some(url) = parts.next() {
		request.with_url(String::from(url));
	}

	request.with_header(key, value);

	Ok(request)
}

#[cfg(test)]
mod tests {
	use crate::{app::HttpMethod, request::HttpRequest};

	use super::*;

	#[test]
	fn test_parse_when_headers_passed() {
		let curl = "
		curl 'https://api.example.com/users/me/' \
		  -H 'Accept: application/json, text/plain, */*' \
		  -H 'Authorization: Bearer some_jwt_token' \
		  -H 'Connection: keep-alive' \
		  -H 'Origin: https://api.example.com' \
		  -H 'Referer: https://api.example.com' \
		  -H 'Sec-Fetch-Site: same-site' \
		";

		assert!(matches!(
			parse_curl(curl),
			Ok(HttpRequest {
				id: String::from("faskfjlkasjflk"),
				created_at: String::from("2023-01-01T00:00:00Z"),
				body: String::from(""),
				method: HttpMethod::Get,
				url: String::from("https://api.example.com/users/me/"),
				headers: vec![
					("Accept", "application/json, text/plain, */*"),
					("Authorization", "Bearer some_jwt_token"),
					("Connection", "keep-alive"),
					("Origin", "https://api.example.com"),
					("Referer", "https://api.example.com"),
					("Sec-Fetch-Site", "same-site"),
				],
			})
		))
	}
}
