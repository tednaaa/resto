use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::http_client::HttpClient;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
	Normal,
	EditingUrl,
	EditingHeaders,
	EditingBody,
	ViewingResponse,
	Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
	Normal,
	Editing,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HttpMethod {
	Get,
	Post,
	Put,
	Delete,
	Patch,
	Head,
	Options,
}

impl HttpMethod {
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Get => "GET",
			Self::Post => "POST",
			Self::Put => "PUT",
			Self::Delete => "DELETE",
			Self::Patch => "PATCH",
			Self::Head => "HEAD",
			Self::Options => "OPTIONS",
		}
	}

	pub const fn next(&self) -> Self {
		match self {
			Self::Get => Self::Post,
			Self::Post => Self::Put,
			Self::Put => Self::Delete,
			Self::Delete => Self::Patch,
			Self::Patch => Self::Head,
			Self::Head => Self::Options,
			Self::Options => Self::Get,
		}
	}

	pub const fn prev(&self) -> Self {
		match self {
			Self::Get => Self::Options,
			Self::Post => Self::Get,
			Self::Put => Self::Post,
			Self::Delete => Self::Put,
			Self::Patch => Self::Delete,
			Self::Head => Self::Patch,
			Self::Options => Self::Head,
		}
	}
}

pub struct App {
	pub state: AppState,
	pub input_mode: InputMode,
	pub current_request: HttpRequest,
	pub responses: Vec<HttpResponse>,
	pub selected_response: Option<usize>,
	pub url_input: String,
	pub headers_input: String,
	pub body_input: String,
	pub cursor_position: usize,
	pub scroll_offset: usize,
	pub http_client: HttpClient,
	pub loading: bool,
	pub error_message: Option<String>,
	pub active_tab: usize, // 0: Request, 1: Response, 2: History
	pub header_edit_index: Option<usize>,
	pub temp_header_key: String,
	pub temp_header_value: String,
}

impl App {
	pub fn new() -> Self {
		Self {
			state: AppState::Normal,
			input_mode: InputMode::Normal,
			current_request: HttpRequest::new(),
			responses: Vec::new(),
			selected_response: None,
			url_input: String::new(),
			headers_input: String::new(),
			body_input: String::new(),
			cursor_position: 0,
			scroll_offset: 0,
			http_client: HttpClient::new(),
			loading: false,
			error_message: None,
			active_tab: 0,
			header_edit_index: None,
			temp_header_key: String::new(),
			temp_header_value: String::new(),
		}
	}

	pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
		match self.input_mode {
			InputMode::Normal => self.handle_normal_mode_key(key).await?,
			InputMode::Editing => self.handle_editing_mode_key(key),
		}
		Ok(())
	}

	async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<()> {
		match key.code {
			KeyCode::Tab => {
				self.active_tab = (self.active_tab + 1) % 3;
			}
			KeyCode::BackTab => {
				self.active_tab = if self.active_tab == 0 { 2 } else { self.active_tab - 1 };
			}
			KeyCode::Char('u') => {
				self.state = AppState::EditingUrl;
				self.input_mode = InputMode::Editing;
				self.url_input = self.current_request.url.clone();
				self.cursor_position = self.url_input.len();
			}
			KeyCode::Char('h') => {
				self.state = AppState::EditingHeaders;
				self.input_mode = InputMode::Editing;
			}
			KeyCode::Char('b') => {
				self.state = AppState::EditingBody;
				self.input_mode = InputMode::Editing;
				self.body_input = self.current_request.body.clone();
				self.cursor_position = self.body_input.len();
			}
			KeyCode::Char('m') => {
				self.current_request.method = self.current_request.method.next();
			}
			KeyCode::Char('M') => {
				self.current_request.method = self.current_request.method.prev();
			}
			KeyCode::Enter => {
				if !self.loading {
					self.send_request().await?;
				}
			}
			KeyCode::Char('r') => {
				self.state = AppState::ViewingResponse;
			}
			KeyCode::Char('?') => {
				self.state = AppState::Help;
			}
			KeyCode::Char('c') => {
				if key.modifiers.contains(KeyModifiers::CONTROL) {
					self.clear_response();
				}
			}
			KeyCode::Up => {
				if self.active_tab == 2 && !self.responses.is_empty() {
					if let Some(selected) = self.selected_response {
						if selected > 0 {
							self.selected_response = Some(selected - 1);
						}
					} else {
						self.selected_response = Some(self.responses.len() - 1);
					}
				}
			}
			KeyCode::Down => {
				if self.active_tab == 2 && !self.responses.is_empty() {
					if let Some(selected) = self.selected_response {
						if selected < self.responses.len() - 1 {
							self.selected_response = Some(selected + 1);
						}
					} else {
						self.selected_response = Some(0);
					}
				}
			}
			_ => {}
		}
		Ok(())
	}

	fn handle_editing_mode_key(&mut self, key: KeyEvent) {
		match key.code {
			KeyCode::Enter => match self.state {
				AppState::EditingUrl => {
					self.current_request.url = self.url_input.clone();
					self.state = AppState::Normal;
					self.input_mode = InputMode::Normal;
				}
				AppState::EditingBody => {
					self.current_request.body = self.body_input.clone();
					self.state = AppState::Normal;
					self.input_mode = InputMode::Normal;
				}
				AppState::EditingHeaders => {
					if !self.temp_header_key.is_empty() && !self.temp_header_value.is_empty() {
						self.current_request
							.headers
							.insert(self.temp_header_key.clone(), self.temp_header_value.clone());
						self.temp_header_key.clear();
						self.temp_header_value.clear();
					}
					self.state = AppState::Normal;
					self.input_mode = InputMode::Normal;
				}
				_ => {}
			},
			KeyCode::Backspace => match self.state {
				AppState::EditingUrl => {
					if self.cursor_position > 0 {
						self.url_input.remove(self.cursor_position - 1);
						self.cursor_position -= 1;
					}
				}
				AppState::EditingBody => {
					if self.cursor_position > 0 {
						self.body_input.remove(self.cursor_position - 1);
						self.cursor_position -= 1;
					}
				}
				AppState::EditingHeaders => {
					if !self.temp_header_value.is_empty() {
						self.temp_header_value.pop();
					} else if !self.temp_header_key.is_empty() {
						self.temp_header_key.pop();
					}
				}
				_ => {}
			},
			KeyCode::Left => {
				if self.cursor_position > 0 {
					self.cursor_position -= 1;
				}
			}
			KeyCode::Right => {
				let max_pos = match self.state {
					AppState::EditingUrl => self.url_input.len(),
					AppState::EditingBody => self.body_input.len(),
					_ => 0,
				};
				if self.cursor_position < max_pos {
					self.cursor_position += 1;
				}
			}
			KeyCode::Char(c) => {
				match self.state {
					AppState::EditingUrl => {
						self.url_input.insert(self.cursor_position, c);
						self.cursor_position += 1;
					}
					AppState::EditingBody => {
						self.body_input.insert(self.cursor_position, c);
						self.cursor_position += 1;
					}
					AppState::EditingHeaders => {
						if c == ':' && self.temp_header_value.is_empty() {
							// Switch from key to value
						} else if self.temp_header_value.is_empty() {
							self.temp_header_key.push(c);
						} else {
							self.temp_header_value.push(c);
						}
					}
					_ => {}
				}
			}
			_ => {}
		}
	}

	async fn send_request(&mut self) -> Result<()> {
		if self.current_request.url.is_empty() {
			self.error_message = Some("URL cannot be empty".to_string());
			return Ok(());
		}

		self.loading = true;
		self.error_message = None;

		match self.http_client.send_request(&self.current_request).await {
			Ok(response) => {
				self.responses.push(response);
				self.selected_response = Some(self.responses.len() - 1);
				self.active_tab = 1; // Switch to response tab
			}
			Err(error) => {
				self.error_message = Some(format!("Request failed: {error}"));
			}
		}

		self.loading = false;
		Ok(())
	}

	// TODO: Handle any background updates here
	#[allow(clippy::unused_async, clippy::needless_pass_by_ref_mut)]
	pub async fn update(&mut self) -> Result<()> {
		Ok(())
	}

	fn clear_response(&mut self) {
		self.responses.clear();
		self.selected_response = None;
	}

	pub fn get_current_response(&self) -> Option<&HttpResponse> {
		self.selected_response
			.map_or_else(|| self.responses.last(), |index| self.responses.get(index))
	}

	pub fn add_header(&mut self, key: String, value: String) {
		self.current_request.headers.insert(key, value);
	}

	pub fn remove_header(&mut self, key: &str) {
		self.current_request.headers.remove(key);
	}
}
