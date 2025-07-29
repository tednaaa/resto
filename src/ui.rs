use ratatui::{
	Frame,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	symbols,
	text::{Line, Span, ToSpan},
	widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Tabs},
};

use crate::{
	app::{App, AppState, FullscreenSection, InputMode},
	response::HttpResponse,
	vim,
};

pub fn draw(frame: &mut Frame, app: &App) {
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Min(0),    // Main content
			Constraint::Length(1), // Footer
		])
		.split(frame.area());

	match app.state {
		AppState::Help => draw_help(frame, chunks[0]),
		_ => draw_main_content(frame, chunks[0], app),
	}

	draw_footer(frame, chunks[1], app);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainContentTab {
	Request,
	History,
}

impl MainContentTab {
	pub const TABS: &'static [Self] = &[Self::Request, Self::History];

	const fn as_str(&self) -> &'static str {
		match self {
			Self::Request => "Request",
			Self::History => "History",
		}
	}

	pub const fn as_index(&self) -> usize {
		match self {
			Self::Request => 0,
			Self::History => 1,
		}
	}

	pub const fn from_index(index: usize) -> Option<Self> {
		match index {
			0 => Some(Self::Request),
			1 => Some(Self::History),
			_ => None,
		}
	}
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &App) {
	let tab_titles: Vec<Line> = MainContentTab::TABS
		.iter()
		.map(|tab| {
			if tab == &app.active_tab {
				Line::from(Span::styled(tab.as_str(), Style::default().fg(Color::Yellow)))
			} else {
				Line::from(Span::styled(tab.as_str(), Style::default().fg(Color::Gray)))
			}
		})
		.collect();

	let tabs_widget = Tabs::new(tab_titles)
		.block(Block::default().padding(Padding::top(1)))
		.divider(symbols::DOT)
		.highlight_style(Style::default().fg(Color::Yellow))
		.select(app.active_tab.as_index());

	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Min(0)])
		.split(area);

	frame.render_widget(tabs_widget, chunks[0]);

	match app.active_tab {
		MainContentTab::Request => draw_request_tab(frame, chunks[1], app),
		MainContentTab::History => draw_history_tab(frame, chunks[1], app),
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestSectionTab {
	Headers,
	Body,
	Query,
}

impl RequestSectionTab {
	pub const TABS: &'static [Self] = &[Self::Headers, Self::Body, Self::Query];

	const fn as_str(&self) -> &'static str {
		match self {
			Self::Headers => "Headers",
			Self::Body => "Body",
			Self::Query => "Query",
		}
	}

	pub const fn as_index(&self) -> usize {
		match self {
			Self::Headers => 0,
			Self::Body => 1,
			Self::Query => 2,
		}
	}

	pub const fn from_index(index: usize) -> Option<Self> {
		match index {
			0 => Some(Self::Headers),
			1 => Some(Self::Body),
			2 => Some(Self::Query),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseSectionTab {
	Body,
	Headers,
	Cookies,
}

impl ResponseSectionTab {
	pub const TABS: &'static [Self] = &[Self::Body, Self::Headers, Self::Cookies];

	const fn as_str(&self) -> &'static str {
		match self {
			Self::Body => "Body",
			Self::Headers => "Headers",
			Self::Cookies => "Cookies",
		}
	}

	pub const fn as_index(&self) -> usize {
		match self {
			Self::Body => 0,
			Self::Headers => 1,
			Self::Cookies => 2,
		}
	}

	pub const fn from_index(index: usize) -> Option<Self> {
		match index {
			0 => Some(Self::Body),
			1 => Some(Self::Headers),
			2 => Some(Self::Cookies),
			_ => None,
		}
	}
}

fn draw_request_tab(frame: &mut Frame, area: Rect, app: &App) {
	let request_section_tab_titles: Vec<Line> = RequestSectionTab::TABS
		.iter()
		.map(|tab| {
			if tab == &app.request_section_active_tab {
				Line::from(Span::styled(tab.as_str(), Style::default().fg(Color::Yellow)))
			} else {
				Line::from(Span::styled(tab.as_str(), Style::default().fg(Color::Gray)))
			}
		})
		.collect();

	let request_section_tabs_widget = Tabs::new(request_section_tab_titles)
		.block(Block::default().borders(Borders::ALL).title("Request"))
		.highlight_style(Style::default().fg(Color::Yellow))
		.select(app.request_section_active_tab.as_index());

	let response_section_tab_titles: Vec<Line> = ResponseSectionTab::TABS
		.iter()
		.map(|tab| {
			if tab == &app.response_section_active_tab {
				Line::from(Span::styled(tab.as_str(), Style::default().fg(Color::Yellow)))
			} else {
				Line::from(Span::styled(tab.as_str(), Style::default().fg(Color::Gray)))
			}
		})
		.collect();

	let response_section_tabs_widget = Tabs::new(response_section_tab_titles)
		.block(Block::default().borders(Borders::ALL).title("Response"))
		.highlight_style(Style::default().fg(Color::Yellow))
		.select(app.response_section_active_tab.as_index());

	let (request_section_height, response_section_height) = match app.fullscreen_section {
		FullscreenSection::None => (Constraint::Percentage(40), Constraint::Percentage(60)),
		FullscreenSection::Request => (Constraint::Percentage(100), Constraint::Percentage(0)),
		FullscreenSection::Response => (Constraint::Percentage(0), Constraint::Percentage(100)),
	};

	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(3),   // Method and URL
			request_section_height,  // Request section
			response_section_height, // Response section
		])
		.split(area);

	let request_section_chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Min(0)])
		.split(chunks[1]);

	let response_section_chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Min(0)])
		.split(chunks[2]);

	draw_method_url_section(frame, chunks[0], app);

	frame.render_widget(request_section_tabs_widget, request_section_chunks[0]);
	match app.request_section_active_tab {
		RequestSectionTab::Headers => draw_request_headers_tab(frame, request_section_chunks[1], app),
		RequestSectionTab::Body => draw_request_body_tab(frame, request_section_chunks[1], app),
		RequestSectionTab::Query => draw_request_queries_tab(frame, request_section_chunks[1], app),
	}

	frame.render_widget(response_section_tabs_widget, response_section_chunks[0]);
	match app.response_section_active_tab {
		ResponseSectionTab::Body => draw_response_body_tab(frame, response_section_chunks[1], app),
		ResponseSectionTab::Headers => draw_response_headers_tab(frame, response_section_chunks[1], app),
		ResponseSectionTab::Cookies => {},
	}
}

fn draw_method_url_section(frame: &mut Frame, area: Rect, app: &App) {
	let method_padding = 6;

	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Length(app.current_request.method.as_str().len() as u16 + method_padding),
			Constraint::Min(0),
		])
		.split(area);

	let method_widget = Paragraph::new(app.current_request.method.as_str())
		.style(Style::default().fg(app.current_request.method.color()).add_modifier(Modifier::BOLD))
		.alignment(Alignment::Center)
		.block(
			Block::default().borders(Borders::ALL).border_style(Style::default().fg(app.current_request.method.color())),
		);
	frame.render_widget(method_widget, chunks[0]);

	if matches!(app.state, AppState::EditingUrl) {
		frame.render_widget(app.get_url_textarea(), chunks[1]);
	} else {
		let url_style = Style::default().fg(Color::White);
		let url_text = if app.current_request.url.is_empty() { "" } else { &app.current_request.url };

		let url_widget = Paragraph::new(url_text).style(url_style).block(
			Block::default()
				.borders(Borders::ALL)
				.title("URL ( press 'u' to edit )")
				.border_style(Style::default().fg(Color::White)),
		);
		frame.render_widget(url_widget, chunks[1]);
	}
}

fn draw_request_headers_tab(frame: &mut Frame, area: Rect, app: &App) {
	if matches!(app.state, AppState::EditingHeaders) {
		frame.render_widget(app.get_headers_textarea(), area);
	} else {
		let headers_text =
			if app.current_request.headers.is_empty() { "" } else { &app.current_request.formatted_headers() };

		let headers_style = Style::default().fg(Color::White);

		let headers_widget = Paragraph::new(headers_text).style(headers_style).block(
			Block::default()
				.borders(Borders::ALL)
				.title("( press 'e' to edit )")
				.padding(Padding::symmetric(2, 1))
				.border_style(Style::default().fg(Color::White)),
		);
		frame.render_widget(headers_widget, area);
	}
}

fn draw_request_body_tab(frame: &mut Frame, area: Rect, app: &App) {
	if matches!(app.state, AppState::EditingBody) {
		frame.render_widget(app.get_body_textarea(), area);
	} else {
		let body_text = if app.current_request.body.is_empty() { "" } else { &app.current_request.body };

		let body_style =
			if app.current_request.has_body() { Style::default().fg(Color::White) } else { Style::default().fg(Color::Gray) };

		let body_widget = Paragraph::new(body_text).style(body_style).block(
			Block::default()
				.borders(Borders::ALL)
				.title("( press 'e' to edit )")
				.padding(Padding::symmetric(2, 1))
				.border_style(Style::default().fg(Color::White)),
		);
		frame.render_widget(body_widget, area);
	}
}

fn draw_request_queries_tab(frame: &mut Frame, area: Rect, app: &App) {
	if matches!(app.state, AppState::EditingQueries) {
		frame.render_widget(app.get_queries_textarea(), area);
	} else {
		let queries_text =
			if app.current_request.queries.is_empty() { "" } else { &app.current_request.formatted_queries() };

		let queries_style = Style::default().fg(Color::White);

		let queries_widget = Paragraph::new(queries_text).style(queries_style).block(
			Block::default()
				.borders(Borders::ALL)
				.title("( press 'e' to edit )")
				.padding(Padding::symmetric(2, 1))
				.border_style(Style::default().fg(Color::White)),
		);
		frame.render_widget(queries_widget, area);
	}
}

fn create_response_block() -> Block<'static> {
	Block::default()
		.padding(Padding::symmetric(2, 1))
		.borders(Borders::ALL)
		.border_style(Style::default().fg(Color::White))
}

fn render_response_content<F>(frame: &mut Frame, area: Rect, app: &App, content_fn: F)
where
	F: FnOnce(&HttpResponse) -> String,
{
	if app.loading {
		let widget = Paragraph::new("loading...")
			.style(Style::default().fg(Color::White))
			.alignment(Alignment::Center)
			.block(create_response_block());
		frame.render_widget(widget, area);
		return;
	}

	if let Some(response) = app.get_current_response() {
		let content = content_fn(response);
		let status_text = app.get_current_response().map_or(String::new(), |response| {
			format!(
				"( {} {} | {} | {}ms )",
				response.status_code,
				response.status_text,
				response.formatted_size(),
				response.response_time
			)
		});

		let widget = Paragraph::new(content).style(Style::default().fg(Color::White)).block(
			create_response_block().title("( press 'r' to inspect )").title(status_text.to_span().into_centered_line()),
		);
		frame.render_widget(widget, area);
	} else {
		let widget = Paragraph::new("No response yet\nSend a request to see the response here")
			.style(Style::default().fg(Color::Gray))
			.alignment(Alignment::Center)
			.block(create_response_block());
		frame.render_widget(widget, area);
	}
}

fn draw_response_body_tab(frame: &mut Frame, area: Rect, app: &App) {
	if matches!(app.state, AppState::InspectingResponseBody) {
		frame.render_widget(app.get_response_body_textarea(), area);
	} else {
		render_response_content(frame, area, app, |response| {
			if response.is_json() {
				response.pretty_json().unwrap_or_else(|_| response.body.clone())
			} else {
				response.body.clone()
			}
		});
	}
}

fn draw_response_headers_tab(frame: &mut Frame, area: Rect, app: &App) {
	if matches!(app.state, AppState::InspectingResponseHeaders) {
		frame.render_widget(app.get_response_headers_textarea(), area);
	} else {
		render_response_content(frame, area, app, HttpResponse::formatted_headers);
	}
}

fn draw_history_tab(frame: &mut Frame, area: Rect, app: &App) {
	if app.responses.is_empty() {
		let no_history = Paragraph::new("No request history\nSend some requests to see them here")
			.style(Style::default().fg(Color::Gray))
			.alignment(Alignment::Center)
			.block(Block::default().borders(Borders::ALL).title("History").border_style(Style::default().fg(Color::White)));
		frame.render_widget(no_history, area);
	} else {
		let items: Vec<ListItem> = app
			.responses
			.iter()
			.enumerate()
			.map(|(i, response)| {
				let content = format!(
					"{} {} - {}ms",
					response.status_code,
					response.created_at.format("%H:%M:%S"),
					response.response_time,
				);

				let style = if Some(i) == app.selected_response {
					Style::default().fg(response.status_color()).add_modifier(Modifier::BOLD)
				} else {
					Style::default().fg(response.status_color())
				};

				ListItem::new(content).style(style)
			})
			.collect();

		let history_list = List::new(items)
			.block(Block::default().borders(Borders::ALL).title("History").border_style(Style::default().fg(Color::White)))
			.highlight_style(Style::default().add_modifier(Modifier::REVERSED));

		frame.render_stateful_widget(history_list, area, &mut ratatui::widgets::ListState::default());
	}
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
	let should_hide_vim_mode = matches!(app.state, AppState::Normal | AppState::Help);

	let vim_mode_text = format!("-- {} --", app.vim.mode);
	let vim_mode_width = if should_hide_vim_mode { 0 } else { vim_mode_text.chars().count() as u16 + 2 };

	let info_text = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
	let info_text_width = info_text.chars().count() as u16;

	let vim_mode_widget =
		Paragraph::new(vim_mode_text).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

	let keybindings_text = match (&app.vim.mode, &app.input_mode) {
		(vim::Mode::Normal, InputMode::Editing) => "Save: Enter | Cancel: Escape",
		(_, InputMode::Normal) => "Help: ? | Switch tabs: Tab | Change method: m/M | Send request: Enter",
		_ => "",
	};

	let mut keybindings_widget = Paragraph::new(keybindings_text).style(Style::default().fg(Color::Yellow));

	if let Some(error) = &app.error_message {
		keybindings_widget = Paragraph::new(format!("Error: {error}")).style(Style::default().fg(Color::Red));
	}

	let info_widget = Paragraph::new(info_text).style(Style::default().fg(Color::Magenta));

	let layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Length(vim_mode_width), Constraint::Min(0), Constraint::Length(info_text_width)])
		.split(area);

	frame.render_widget(vim_mode_widget, layout[0]);
	frame.render_widget(keybindings_widget, layout[1]);
	frame.render_widget(info_widget, layout[2]);
}

fn draw_help(frame: &mut Frame, area: Rect) {
	let help_text = vec![
		"Navigation:",
		"  Tab - Iterate through tabs",
		"  ]   - Iterate through request tabs",
		"  }   - Iterate through response tabs",
		"  q   - Quit application",
		"",
		"Request Building:",
		"  u             - Edit URL",
		"  e             - Edit focused request headers/body ..etc",
		"  r             - Inspect focused response headers/body ..etc",
		"  m/M           - Change HTTP method (forward/backward)",
		"  Enter         - Send request",
		"",
		"Press Esc to close this help screen.",
	];

	let help_paragraph = Paragraph::new(help_text.join("\n"))
		.style(Style::default().fg(Color::White))
		.block(Block::default().borders(Borders::ALL).title("Help").border_style(Style::default().fg(Color::Yellow)));
	frame.render_widget(help_paragraph, area);
}

#[allow(dead_code)]
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
	let popup_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Percentage((100 - percent_y) / 2),
			Constraint::Percentage(percent_y),
			Constraint::Percentage((100 - percent_y) / 2),
		])
		.split(r);

	Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage((100 - percent_x) / 2),
			Constraint::Percentage(percent_x),
			Constraint::Percentage((100 - percent_x) / 2),
		])
		.split(popup_layout[1])[1]
}
