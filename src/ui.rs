use ratatui::{
	Frame,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
};

use crate::app::{App, AppState, InputMode};

pub fn draw(f: &mut Frame, app: &App) {
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(3), // Header
			Constraint::Min(0),    // Main content
			Constraint::Length(1), // Footer
		])
		.split(f.area());

	draw_header(f, chunks[0], app);

	match app.state {
		AppState::Help => draw_help(f, chunks[1]),
		_ => draw_main_content(f, chunks[1], app),
	}

	draw_footer(f, chunks[2], app);

	if matches!(app.input_mode, InputMode::Editing) {
		draw_input_popup(f, app);
	}

	if app.loading {
		draw_loading_popup(f);
	}
}

fn draw_header(f: &mut Frame, area: Rect, _app: &App) {
	let title = "resto - HTTP Client";
	let header = Paragraph::new(title)
		.style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
		.alignment(Alignment::Center)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.border_style(Style::default().fg(Color::White)),
		);
	f.render_widget(header, area);
}

fn draw_main_content(f: &mut Frame, area: Rect, app: &App) {
	let tabs = ["Request", "Response", "History"];
	let tab_titles: Vec<Line> = tabs
		.iter()
		.enumerate()
		.map(|(i, &tab)| {
			if i == app.active_tab {
				Line::from(Span::styled(tab, Style::default().fg(Color::Yellow)))
			} else {
				Line::from(Span::styled(tab, Style::default().fg(Color::Gray)))
			}
		})
		.collect();

	let tabs_widget = Tabs::new(tab_titles)
		.block(Block::default().borders(Borders::ALL).title("Tabs"))
		.highlight_style(Style::default().fg(Color::Yellow))
		.select(app.active_tab);

	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Min(0)])
		.split(area);

	f.render_widget(tabs_widget, chunks[0]);

	match app.active_tab {
		0 => draw_request_tab(f, chunks[1], app),
		1 => draw_response_tab(f, chunks[1], app),
		2 => draw_history_tab(f, chunks[1], app),
		_ => {}
	}
}

fn draw_request_tab(f: &mut Frame, area: Rect, app: &App) {
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(3), // Method and URL
			Constraint::Length(8), // Headers
			Constraint::Min(0),    // Body
		])
		.split(area);

	draw_method_url_section(f, chunks[0], app);

	draw_headers_section(f, chunks[1], app);

	draw_body_section(f, chunks[2], app);
}

fn draw_method_url_section(f: &mut Frame, area: Rect, app: &App) {
	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Length(10), Constraint::Min(0)])
		.split(area);

	let method_style = if matches!(app.state, AppState::Normal) {
		Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
	} else {
		Style::default().fg(Color::Gray)
	};

	let method_widget = Paragraph::new(app.current_request.method.as_str())
		.style(method_style)
		.alignment(Alignment::Center)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Method")
				.border_style(Style::default().fg(Color::White)),
		);
	f.render_widget(method_widget, chunks[0]);

	let url_style = if matches!(app.state, AppState::EditingUrl) {
		Style::default().fg(Color::Yellow)
	} else {
		Style::default().fg(Color::White)
	};

	let url_text = if app.current_request.url.is_empty() {
		"Enter URL (press 'u' to edit)"
	} else {
		&app.current_request.url
	};

	let url_widget = Paragraph::new(url_text).style(url_style).block(
		Block::default()
			.borders(Borders::ALL)
			.title("URL")
			.border_style(Style::default().fg(Color::White)),
	);
	f.render_widget(url_widget, chunks[1]);
}

fn draw_headers_section(f: &mut Frame, area: Rect, app: &App) {
	let headers_text = if app.current_request.headers.is_empty() {
		"No headers (press 'h' to add)"
	} else {
		&app.current_request.formatted_headers()
	};

	let headers_style = if matches!(app.state, AppState::EditingHeaders) {
		Style::default().fg(Color::Yellow)
	} else {
		Style::default().fg(Color::White)
	};

	let headers_widget = Paragraph::new(headers_text)
		.style(headers_style)
		.wrap(Wrap { trim: true })
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Headers")
				.border_style(Style::default().fg(Color::White)),
		);
	f.render_widget(headers_widget, area);
}

fn draw_body_section(f: &mut Frame, area: Rect, app: &App) {
	let body_text = if app.current_request.body.is_empty() {
		if app.current_request.has_body() {
			"Request body (press 'b' to edit)"
		} else {
			"No body for this method"
		}
	} else {
		&app.current_request.body
	};

	let body_style = if matches!(app.state, AppState::EditingBody) {
		Style::default().fg(Color::Yellow)
	} else if app.current_request.has_body() {
		Style::default().fg(Color::White)
	} else {
		Style::default().fg(Color::Gray)
	};

	let body_widget = Paragraph::new(body_text)
		.style(body_style)
		.wrap(Wrap { trim: true })
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Body")
				.border_style(Style::default().fg(Color::White)),
		);
	f.render_widget(body_widget, area);
}

fn draw_response_tab(f: &mut Frame, area: Rect, app: &App) {
	if let Some(response) = app.get_current_response() {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Length(3), // Status line
				Constraint::Length(8), // Headers
				Constraint::Min(0),    // Body
			])
			.split(area);

		let status_color = match response.status_code {
			200..=299 => Color::Green,
			300..=399 => Color::Yellow,
			400..=499 => Color::Red,
			500..=599 => Color::Magenta,
			_ => Color::White,
		};

		let status_text = format!(
			"{} {} | {} | {}ms",
			response.status_code,
			response.status_text,
			response.formatted_size(),
			response.response_time
		);

		let status_widget = Paragraph::new(status_text)
			.style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
			.alignment(Alignment::Left)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("Status")
					.border_style(Style::default().fg(Color::White)),
			);
		f.render_widget(status_widget, chunks[0]);

		let headers_widget = Paragraph::new(response.formatted_headers())
			.style(Style::default().fg(Color::White))
			.wrap(Wrap { trim: true })
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("Response Headers")
					.border_style(Style::default().fg(Color::White)),
			);
		f.render_widget(headers_widget, chunks[1]);

		let body_text = if response.is_json() {
			response.pretty_json().unwrap_or_else(|_| response.body.clone())
		} else {
			response.body.clone()
		};

		let body_widget = Paragraph::new(body_text)
			.style(Style::default().fg(Color::White))
			.wrap(Wrap { trim: true })
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("Response Body")
					.border_style(Style::default().fg(Color::White)),
			);
		f.render_widget(body_widget, chunks[2]);
	} else {
		let no_response = Paragraph::new("No response yet\nSend a request to see the response here")
			.style(Style::default().fg(Color::Gray))
			.alignment(Alignment::Center)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("Response")
					.border_style(Style::default().fg(Color::White)),
			);
		f.render_widget(no_response, area);
	}
}

fn draw_history_tab(f: &mut Frame, area: Rect, app: &App) {
	if app.responses.is_empty() {
		let no_history = Paragraph::new("No request history\nSend some requests to see them here")
			.style(Style::default().fg(Color::Gray))
			.alignment(Alignment::Center)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("History")
					.border_style(Style::default().fg(Color::White)),
			);
		f.render_widget(no_history, area);
	} else {
		let items: Vec<ListItem> = app
			.responses
			.iter()
			.enumerate()
			.map(|(i, response)| {
				let status_color = match response.status_code {
					200..=299 => Color::Green,
					300..=399 => Color::Yellow,
					400..=499 => Color::Red,
					500..=599 => Color::Magenta,
					_ => Color::White,
				};

				let content = format!(
					"{} {} - {}ms",
					response.status_code,
					response.created_at.format("%H:%M:%S"),
					response.response_time
				);

				let style = if Some(i) == app.selected_response {
					Style::default().fg(status_color).add_modifier(Modifier::BOLD)
				} else {
					Style::default().fg(status_color)
				};

				ListItem::new(content).style(style)
			})
			.collect();

		let history_list = List::new(items)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("History")
					.border_style(Style::default().fg(Color::White)),
			)
			.highlight_style(Style::default().add_modifier(Modifier::REVERSED));

		f.render_stateful_widget(history_list, area, &mut ratatui::widgets::ListState::default());
	}
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
	let mut help_text = vec![
		Span::raw("Help: ? | "),
		Span::raw("Switch tabs: Tab | "),
		Span::raw("Change method: m/M | "),
		Span::raw("Send request: Enter"),
	];

	if let Some(error) = &app.error_message {
		help_text = vec![Span::styled(format!("Error: {error}"), Style::default().fg(Color::Red))];
	}

	let footer = Paragraph::new(Line::from(help_text))
		.style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
		.alignment(Alignment::Left);
	f.render_widget(footer, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
	let help_text = vec![
		"resto - HTTP Client Help",
		"",
		"Navigation:",
		"  Tab/Shift+Tab  - Switch between tabs",
		"  ↑/↓           - Navigate history (in History tab)",
		"  Esc           - Cancel current action/go back",
		"  q             - Quit application",
		"",
		"Request Building:",
		"  u             - Edit URL",
		"  h             - Edit headers",
		"  b             - Edit body",
		"  m/M           - Change HTTP method (forward/backward)",
		"  Enter         - Send request",
		"",
		"Other:",
		"  r             - View response",
		"  Ctrl+C        - Clear response",
		"  ?             - Show/hide this help",
		"",
		"Press Esc to close this help screen.",
	];

	let help_paragraph = Paragraph::new(help_text.join("\n"))
		.style(Style::default().fg(Color::White))
		.wrap(Wrap { trim: true })
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Help")
				.border_style(Style::default().fg(Color::Yellow)),
		);
	f.render_widget(help_paragraph, area);
}

fn draw_input_popup(f: &mut Frame, app: &App) {
	let popup_area = centered_rect(60, 20, f.area());

	f.render_widget(Clear, popup_area);

	let header_input;
	let (title, content) = match app.state {
		AppState::EditingUrl => ("Edit URL", &app.url_input),
		AppState::EditingBody => ("Edit Body", &app.body_input),
		AppState::EditingHeaders => {
			header_input = format!("{}:{}", app.temp_header_key, app.temp_header_value);
			("Add Header (key:value)", &header_input)
		}
		_ => {
			header_input = String::new();
			("Input", &header_input)
		}
	};

	let popup = Paragraph::new(content.clone())
		.style(Style::default().fg(Color::Yellow))
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title(title)
				.border_style(Style::default().fg(Color::Yellow)),
		);
	f.render_widget(popup, popup_area);
}

fn draw_loading_popup(f: &mut Frame) {
	let popup_area = centered_rect(30, 10, f.area());

	f.render_widget(Clear, popup_area);

	let loading = Paragraph::new("Sending request...")
		.style(Style::default().fg(Color::Yellow))
		.alignment(Alignment::Center)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Loading")
				.border_style(Style::default().fg(Color::Yellow)),
		);
	f.render_widget(loading, popup_area);
}

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
