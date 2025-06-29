use crate::{app::HttpMethod, request::HttpRequest};

#[derive(Debug)]
pub enum CurlParseError {
	InvalidFormat(String),
	MissingUrl,
	InvalidMethod(String),
	InvalidHeader(String),
}

impl std::fmt::Display for CurlParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidFormat(message) => write!(f, "Invalid curl format: {message}"),
			Self::MissingUrl => write!(f, "Missing URL in curl command"),
			Self::InvalidMethod(method) => write!(f, "Invalid HTTP method: {method}"),
			Self::InvalidHeader(header) => write!(f, "Invalid header format: {header}"),
		}
	}
}

impl std::error::Error for CurlParseError {}

pub fn parse_curl(input: &str) -> anyhow::Result<HttpRequest> {
	let mut input = input.trim();

	if let Some(inp) = input.strip_prefix("curl ") {
		input = inp.trim();
	}

	let mut request = HttpRequest::new();
	let tokens = tokenize_curl_command(input)?;
	let mut i = 0;

	while i < tokens.len() {
		let token = &tokens[i];

		match token.as_str() {
			"-X" | "--request" => {
				i += 1;
				if i >= tokens.len() {
					return Err(CurlParseError::InvalidFormat("Missing method after -X".to_string()).into());
				}
				let method = tokens[i].parse::<HttpMethod>().map_err(|_| CurlParseError::InvalidMethod(tokens[i].clone()))?;
				request = request.with_method(method);
			},
			"-H" | "--header" => {
				i += 1;
				if i >= tokens.len() {
					return Err(CurlParseError::InvalidFormat("Missing header after -H".to_string()).into());
				}
				let header_str = &tokens[i];
				if let Some(colon_pos) = header_str.find(':') {
					let key = header_str[..colon_pos].trim().to_string();
					let value = header_str[colon_pos + 1..].trim().to_string();
					request = request.with_header(key, value);
				} else {
					return Err(CurlParseError::InvalidHeader(header_str.clone()).into());
				}
			},
			"-d" | "--data" | "--data-raw" => {
				i += 1;
				if i >= tokens.len() {
					return Err(CurlParseError::InvalidFormat("Missing data after -d".to_string()).into());
				}
				request.set_body(&tokens[i])?;
				if matches!(request.method, HttpMethod::Get) {
					request = request.with_method(HttpMethod::Post);
				}
			},
			"--data-binary" => {
				i += 1;
				if i >= tokens.len() {
					return Err(CurlParseError::InvalidFormat("Missing data after --data-binary".to_string()).into());
				}
				request.set_body(&tokens[i])?;
				if matches!(request.method, HttpMethod::Get) {
					request = request.with_method(HttpMethod::Post);
				}
			},
			"--compressed" | "-L" | "--location" | "-k" | "--insecure" | "-s" | "--silent" | "-v" | "--verbose" => {
				// Skip common curl flags that don't affect the HTTP request structure
			},
			_ => {
				if token.starts_with("http") {
					let url = token.clone();

					if let Some(query_start) = url.find('?') {
						let base_url = url[..query_start].to_string();
						let query_str = &url[query_start + 1..];

						for query_pair in query_str.split('&') {
							if let Some(eq_pos) = query_pair.find('=') {
								let key = query_pair[..eq_pos].to_string();
								let value = query_pair[eq_pos + 1..].to_string();
								request = request.with_query(key, value);
							} else if !query_pair.is_empty() {
								request = request.with_query(query_pair.to_string(), String::new());
							}
						}

						request = request.with_url(base_url);
					} else {
						request = request.with_url(url);
					}
				}
			},
		}
		i += 1;
	}

	if request.url.is_empty() {
		return Err(CurlParseError::MissingUrl.into());
	}

	Ok(request)
}

fn tokenize_curl_command(input: &str) -> anyhow::Result<Vec<String>> {
	let mut tokens = Vec::new();
	let mut current_token = String::new();
	let mut in_quotes = false;
	let mut quote_char = '"';
	let mut escape_next = false;

	for ch in input.chars() {
		if escape_next {
			current_token.push(ch);
			escape_next = false;
			continue;
		}

		match ch {
			'\\' => {
				escape_next = true;
			},
			'"' | '\'' => {
				if !in_quotes {
					in_quotes = true;
					quote_char = ch;
				} else if ch == quote_char {
					in_quotes = false;
				} else {
					current_token.push(ch);
				}
			},
			' ' | '\t' | '\n' | '\r' => {
				if in_quotes {
					current_token.push(ch);
				} else if !current_token.is_empty() {
					tokens.push(current_token.clone());
					current_token.clear();
				}
			},
			_ => {
				current_token.push(ch);
			},
		}
	}

	if !current_token.is_empty() {
		tokens.push(current_token);
	}

	if in_quotes {
		return Err(CurlParseError::InvalidFormat("Unclosed quotes in curl command".to_string()).into());
	}

	Ok(tokens)
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use serde_json::json;

	use super::*;

	#[test]
	fn test_parse_when_only_url_passed() {
		let curl = "curl 'http://api.example.com/users/me/'";

		let result = parse_curl(curl).unwrap();

		assert_eq!(result.url, "http://api.example.com/users/me/");
		assert_eq!(result.method, HttpMethod::Get);
	}

	#[test]
	fn test_parse_when_headers_passed() {
		let curl = r#"
			curl "https://api.example.com/users/me/" \
			  -H 'Accept: application/json, text/plain, */*' \
			  -H 'Authorization: Bearer some_jwt_token' \
			  -H 'Connection: keep-alive' \
			  -H 'Origin: https://api.example.com' \
			  -H 'Referer: https://api.example.com' \
			  -H 'Sec-Fetch-Site: same-site'
		"#;

		let result = parse_curl(curl).unwrap();

		assert_eq!(result.url, "https://api.example.com/users/me/");
		assert_eq!(result.method, HttpMethod::Get);

		assert_eq!(
			result.headers,
			HashMap::from([
				(String::from("Accept"), String::from("application/json, text/plain, */*")),
				(String::from("Authorization"), String::from("Bearer some_jwt_token")),
				(String::from("Connection"), String::from("keep-alive")),
				(String::from("Origin"), String::from("https://api.example.com")),
				(String::from("Referer"), String::from("https://api.example.com")),
				(String::from("Sec-Fetch-Site"), String::from("same-site")),
			])
		);
	}

	#[test]
	fn test_when_data_raw_passed() {
		let curl = r#"
			curl 'https://api.example.ru/api/projects/' \
			  -H 'Accept: application/json, text/plain, */*' \
			  -H 'Content-Type: application/json' \
			  --data-raw '{"name":"test","projectType":"DAILY","isActive":false}'
		"#;

		let result = parse_curl(curl).unwrap();

		assert_eq!(result.url, "https://api.example.ru/api/projects/");
		assert_eq!(result.method, HttpMethod::Post);

		assert_eq!(
			result.headers,
			HashMap::from([
				(String::from("Accept"), String::from("application/json, text/plain, */*")),
				(String::from("Content-Type"), String::from("application/json")),
			])
		);

		let expected_body = serde_json::to_string_pretty(&json!({
				"isActive": false,
				"name": "test",
				"projectType": "DAILY"
		}))
		.unwrap();
		assert_eq!(result.body, expected_body);
	}

	#[test]
	fn test_when_queries_passed() {
		let curl = r"
			curl 'https://api.example.ru/api/projects/?name=&ordering=-index&limit=50&offset=0&is_hidden=false' \
			  -H 'Accept: application/json, text/plain, */*' \
			  -H 'Connection: keep-alive' \
			  -H 'x-use-camel-case: true'
		";

		let result = parse_curl(curl).unwrap();

		assert_eq!(result.url, "https://api.example.ru/api/projects/");
		assert_eq!(result.method, HttpMethod::Get);

		assert_eq!(
			result.headers,
			HashMap::from([
				(String::from("Accept"), String::from("application/json, text/plain, */*")),
				(String::from("Connection"), String::from("keep-alive")),
				(String::from("x-use-camel-case"), String::from("true")),
			])
		);

		assert_eq!(
			result.queries,
			HashMap::from([
				(String::from("name"), String::new()),
				(String::from("ordering"), String::from("-index")),
				(String::from("limit"), String::from("50")),
				(String::from("offset"), String::from("0")),
				(String::from("is_hidden"), String::from("false")),
			])
		);
	}

	#[test]
	fn test_when_delete_method_passed() {
		let curl = r"
			curl 'https://api.example.ru/api/projects/6a3c6b7d/' \
			  -X 'DELETE' \
			  -H 'Accept: application/json, text/plain, */*' \
			  -H 'x-use-camel-case: true'
		";

		let result = parse_curl(curl).unwrap();

		assert_eq!(result.url, "https://api.example.ru/api/projects/6a3c6b7d/");
		assert_eq!(result.method, HttpMethod::Delete);

		assert_eq!(
			result.headers,
			HashMap::from([
				(String::from("Accept"), String::from("application/json, text/plain, */*")),
				(String::from("x-use-camel-case"), String::from("true")),
			])
		);

		assert_eq!(result.body, "");
	}
}
