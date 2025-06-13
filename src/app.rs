use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use tui_textarea::{Input, TextArea};

use crate::curl::parse_curl;
use crate::http_client::HttpClient;
use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::ui::{MainContentTab, RequestSectionTab, ResponseSectionTab};
use crate::vim::{Mode, Transition, Vim};

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

impl std::str::FromStr for HttpMethod {
	type Err = String;

	fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
		match s.to_uppercase().as_str() {
			"GET" => Ok(Self::Get),
			"POST" => Ok(Self::Post),
			"PUT" => Ok(Self::Put),
			"DELETE" => Ok(Self::Delete),
			"PATCH" => Ok(Self::Patch),
			"HEAD" => Ok(Self::Head),
			"OPTIONS" => Ok(Self::Options),
			_ => Err(format!("Unknown HTTP method: {s}")),
		}
	}
}

impl HttpMethod {
	pub const fn as_str(&self) -> &str {
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

	pub const fn color(&self) -> Color {
		match self {
			Self::Get => Color::Green,
			Self::Post => Color::Blue,
			Self::Put => Color::Magenta,
			Self::Patch => Color::LightMagenta,
			Self::Delete => Color::Red,
			Self::Head => Color::Cyan,
			Self::Options => Color::Yellow,
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

	pub const fn previous(&self) -> Self {
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

	pub url_textarea: TextArea<'static>,
	pub headers_textarea: TextArea<'static>,
	pub body_textarea: TextArea<'static>,

	pub http_client: HttpClient,
	pub loading: bool,
	pub error_message: Option<String>,

	pub active_tab: MainContentTab,
	pub request_section_active_tab: RequestSectionTab,
	pub response_section_active_tab: ResponseSectionTab,

	pub vim: Vim,
}

impl App {
	pub fn new() -> Self {
		let url_textarea = TextArea::default();
		let headers_textarea = TextArea::default();
		let body_textarea = TextArea::default();

		let vim = Vim::new(Mode::Normal);

		Self {
			state: AppState::Normal,
			input_mode: InputMode::Normal,
			current_request: HttpRequest::new(),
			responses: Vec::new(),
			selected_response: None,
			url_textarea,
			headers_textarea,
			body_textarea,
			http_client: HttpClient::new(),
			loading: false,
			error_message: None,
			active_tab: MainContentTab::Request,
			request_section_active_tab: RequestSectionTab::Headers,
			response_section_active_tab: ResponseSectionTab::Body,
			vim,
		}
	}

	fn next_tab(&mut self) {
		let next_index = (self.active_tab.as_index() + 1) % MainContentTab::TABS.len();
		self.active_tab = MainContentTab::from_index(next_index).unwrap_or(MainContentTab::Request);
	}

	fn previous_tab(&mut self) {
		let current_index = self.active_tab.as_index();
		let previous_index = if current_index == 0 {
			MainContentTab::TABS.len() - 1
		} else {
			current_index - 1
		};
		self.active_tab = MainContentTab::from_index(previous_index).unwrap_or(MainContentTab::Request);
	}

	fn request_section_next_tab(&mut self) {
		let next_index = (self.request_section_active_tab.as_index() + 1) % RequestSectionTab::TABS.len();
		self.request_section_active_tab = RequestSectionTab::from_index(next_index).unwrap_or(RequestSectionTab::Body);
	}

	fn request_section_previous_tab(&mut self) {
		let previous_index = if self.request_section_active_tab.as_index() == 0 {
			RequestSectionTab::TABS.len() - 1
		} else {
			self.request_section_active_tab.as_index() - 1
		};
		self.request_section_active_tab = RequestSectionTab::from_index(previous_index).unwrap_or(RequestSectionTab::Body);
	}

	fn response_section_next_tab(&mut self) {
		let next_index = (self.response_section_active_tab.as_index() + 1) % ResponseSectionTab::TABS.len();
		self.response_section_active_tab = ResponseSectionTab::from_index(next_index).unwrap_or(ResponseSectionTab::Headers);
	}

	fn response_section_previous_tab(&mut self) {
		let previous_index = if self.response_section_active_tab.as_index() == 0 {
			ResponseSectionTab::TABS.len() - 1
		} else {
			self.response_section_active_tab.as_index() - 1
		};
		self.response_section_active_tab = ResponseSectionTab::from_index(previous_index).unwrap_or(ResponseSectionTab::Headers);
	}

	pub async fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
		match self.input_mode {
			InputMode::Normal => self.handle_normal_mode_key(key).await,
			InputMode::Editing => self.handle_editing_mode_key(key),
		}
	}

	pub fn handle_paste(&mut self, text: String) -> anyhow::Result<()> {
		if self.state == AppState::EditingUrl {
			self.url_textarea.insert_str(text);
			self.save_current_textarea_content()?;
			self.state = AppState::Normal;
			self.input_mode = InputMode::Normal;
			self.vim = Vim::new(Mode::Normal);
		}

		Ok(())
	}

	async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
		match key.code {
			KeyCode::Char('q') => {
				return Ok(true); // Signal quit
			}
			KeyCode::Tab => self.next_tab(),
			KeyCode::BackTab => self.previous_tab(),
			KeyCode::Char(']') => self.request_section_next_tab(),
			KeyCode::Char('[') => self.request_section_previous_tab(),
			KeyCode::Char('}') => self.response_section_next_tab(),
			KeyCode::Char('{') => self.response_section_previous_tab(),
			KeyCode::Char('u') => {
				self.state = AppState::EditingUrl;
				self.input_mode = InputMode::Editing;
				self.url_textarea = TextArea::from([self.current_request.url.as_str()]);

				if self.current_request.url.is_empty() {
					self.vim = Vim::new(Mode::Insert);
				} else {
					self.vim = Vim::new(Mode::Normal);
				}

				self.setup_textarea_for_vim();
			}
			KeyCode::Char('h') => {
				self.state = AppState::EditingHeaders;
				self.input_mode = InputMode::Editing;

				let headers_text = self.current_request.formatted_headers();

				self.headers_textarea = if headers_text.is_empty() {
					self.vim = Vim::new(Mode::Insert);
					TextArea::default()
				} else {
					self.vim = Vim::new(Mode::Normal);
					TextArea::from(headers_text.lines().collect::<Vec<_>>())
				};

				self.setup_textarea_for_vim();
			}
			KeyCode::Char('b') => {
				self.state = AppState::EditingBody;
				self.input_mode = InputMode::Editing;

				self.body_textarea = if self.current_request.body.is_empty() {
					self.vim = Vim::new(Mode::Insert);
					TextArea::default()
				} else {
					self.vim = Vim::new(Mode::Normal);
					TextArea::from(self.current_request.body.lines().collect::<Vec<_>>())
				};

				self.setup_textarea_for_vim();
			}
			KeyCode::Char('m') => {
				self.current_request.method = self.current_request.method.next();
			}
			KeyCode::Char('M') => {
				self.current_request.method = self.current_request.method.previous();
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
				if self.active_tab == MainContentTab::History && !self.responses.is_empty() {
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
				if self.active_tab == MainContentTab::History && !self.responses.is_empty() {
					if let Some(selected) = self.selected_response {
						if selected < self.responses.len() - 1 {
							self.selected_response = Some(selected + 1);
						}
					} else {
						self.selected_response = Some(0);
					}
				}
			}
			KeyCode::Esc => {
				if matches!(self.state, AppState::Help | AppState::ViewingResponse) {
					self.state = AppState::Normal;
				}
			}
			_ => {}
		}
		Ok(false)
	}

	fn handle_editing_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
		if self.vim.mode == Mode::Normal {
			match key.code {
				KeyCode::Enter => {
					self.save_current_textarea_content()?;
					self.state = AppState::Normal;
					self.input_mode = InputMode::Normal;
					return Ok(false);
				}
				KeyCode::Esc => {
					self.state = AppState::Normal;
					self.input_mode = InputMode::Normal;
					return Ok(false);
				}
				_ => {}
			}
		}

		let input: Input = key.into();

		let textarea = match self.state {
			AppState::EditingUrl => &mut self.url_textarea,
			AppState::EditingHeaders => &mut self.headers_textarea,
			AppState::EditingBody => &mut self.body_textarea,
			_ => return Ok(false),
		};

		match self.vim.transition(input, textarea) {
			Transition::Mode(mode) if self.vim.mode != mode => {
				let title = match self.state {
					AppState::EditingUrl => "URL",
					AppState::EditingHeaders => "Headers",
					AppState::EditingBody => "Body",
					_ => return Ok(false),
				};

				textarea.set_block(mode.block(title));
				textarea.set_cursor_style(mode.cursor_style());
				self.vim = Vim::new(mode);
			}
			Transition::Nop | Transition::Mode(_) => {}
			Transition::Pending(pending_input) => {
				self.vim = self.vim.clone().with_pending(pending_input);
			}
			Transition::Quit => {
				self.state = AppState::Normal;
				self.input_mode = InputMode::Normal;
				self.vim = Vim::new(Mode::Normal);
			}
		}

		Ok(false)
	}

	fn save_current_textarea_content(&mut self) -> anyhow::Result<()> {
		match self.state {
			AppState::EditingUrl => {
				let url_text = self.url_textarea.lines().join("").trim().to_owned();

				if url_text.starts_with("curl") {
					self.current_request = parse_curl(&url_text)?;
				} else {
					self.current_request.url = url_text;
				}
			}
			AppState::EditingHeaders => {
				self.current_request.headers.clear();
				for line in self.headers_textarea.lines() {
					if let Some((key, value)) = line.split_once(':') {
						let key = key.trim().to_string();
						let value = value.trim().to_string();
						if !key.is_empty() && !value.is_empty() {
							self.current_request.headers.insert(key, value);
						}
					}
				}
			}
			AppState::EditingBody => {
				self.current_request.body = self.body_textarea.lines().join("\n");
			}
			_ => {}
		}

		Ok(())
	}

	fn setup_textarea_for_vim(&mut self) {
		let textarea = match self.state {
			AppState::EditingUrl => &mut self.url_textarea,
			AppState::EditingHeaders => &mut self.headers_textarea,
			AppState::EditingBody => &mut self.body_textarea,
			_ => return,
		};

		let title = match self.state {
			AppState::EditingUrl => "URL",
			AppState::EditingHeaders => "Headers",
			AppState::EditingBody => "Body",
			_ => return,
		};

		match self.state {
			AppState::EditingUrl => {
				textarea.set_placeholder_text("Enter URL... or paste curl");
			}
			AppState::EditingHeaders => {
				textarea.set_line_number_style(Style::default().bg(Color::DarkGray));
				textarea.set_placeholder_text("key: value\n\nkey2: value2");
			}
			AppState::EditingBody => {
				textarea.set_line_number_style(Style::default().bg(Color::DarkGray));
				textarea.set_placeholder_text("Request body (JSON, text, etc.)");
			}
			_ => {}
		}

		textarea.set_tab_length(2);
		textarea.set_block(self.vim.mode.block(title));
		textarea.set_cursor_style(self.vim.mode.cursor_style());
	}

	async fn send_request(&mut self) -> anyhow::Result<()> {
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
	pub async fn update(&mut self) -> anyhow::Result<()> {
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

	pub const fn get_url_textarea(&self) -> &TextArea<'static> {
		&self.url_textarea
	}

	pub const fn get_headers_textarea(&self) -> &TextArea<'static> {
		&self.headers_textarea
	}

	pub const fn get_body_textarea(&self) -> &TextArea<'static> {
		&self.body_textarea
	}
}
