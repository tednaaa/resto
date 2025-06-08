use ratatui::{
	Frame,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
};

use crate::app::{App, AppState, InputMode};

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

	if app.loading {
		draw_loading_popup(frame);
	}
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &App) {
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

	frame.render_widget(tabs_widget, chunks[0]);

	match app.active_tab {
		0 => draw_request_tab(frame, chunks[1], app),
		1 => draw_response_tab(frame, chunks[1], app),
		2 => draw_history_tab(frame, chunks[1], app),
		_ => {}
	}
}

fn draw_request_tab(frame: &mut Frame, area: Rect, app: &App) {
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(3), // Method and URL
			Constraint::Length(8), // Headers
			Constraint::Min(0),    // Body
		])
		.split(area);

	draw_method_url_section(frame, chunks[0], app);
	draw_headers_section(frame, chunks[1], app);
	draw_body_section(frame, chunks[2], app);
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
		.style(
			Style::default()
				.fg(app.current_request.method.color())
				.add_modifier(Modifier::BOLD),
		)
		.alignment(Alignment::Center)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Method")
				.border_style(Style::default().fg(Color::White)),
		);
	frame.render_widget(method_widget, chunks[0]);

	if matches!(app.state, AppState::EditingUrl) {
		frame.render_widget(app.get_url_textarea(), chunks[1]);
	} else {
		let url_style = Style::default().fg(Color::White);
		let url_text = if app.current_request.url.is_empty() {
			""
		} else {
			&app.current_request.url
		};

		let url_widget = Paragraph::new(url_text).style(url_style).block(
			Block::default()
				.borders(Borders::ALL)
				.title("URL (press 'u' to edit) ")
				.border_style(Style::default().fg(Color::White)),
		);
		frame.render_widget(url_widget, chunks[1]);
	}
}

fn draw_headers_section(frame: &mut Frame, area: Rect, app: &App) {
	if matches!(app.state, AppState::EditingHeaders) {
		frame.render_widget(app.get_headers_textarea(), area);
	} else {
		let headers_text = if app.current_request.headers.is_empty() {
			""
		} else {
			&app.current_request.formatted_headers()
		};

		let headers_style = Style::default().fg(Color::White);

		let headers_widget = Paragraph::new(headers_text)
			.style(headers_style)
			.wrap(Wrap { trim: true })
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("Headers (press 'h' to edit) ")
					.border_style(Style::default().fg(Color::White)),
			);
		frame.render_widget(headers_widget, area);
	}
}

fn draw_body_section(frame: &mut Frame, area: Rect, app: &App) {
	if matches!(app.state, AppState::EditingBody) {
		frame.render_widget(app.get_body_textarea(), area);
	} else {
		let body_text = if app.current_request.body.is_empty() {
			""
		} else {
			&app.current_request.body
		};

		let body_style = if app.current_request.has_body() {
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
					.title("Body (press 'b' to edit) ")
					.border_style(Style::default().fg(Color::White)),
			);
		frame.render_widget(body_widget, area);
	}
}

fn draw_response_tab(frame: &mut Frame, area: Rect, app: &App) {
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
		frame.render_widget(status_widget, chunks[0]);

		let headers_widget = Paragraph::new(response.formatted_headers())
			.style(Style::default().fg(Color::White))
			.wrap(Wrap { trim: true })
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title("Response Headers")
					.border_style(Style::default().fg(Color::White)),
			);
		frame.render_widget(headers_widget, chunks[1]);

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
		frame.render_widget(body_widget, chunks[2]);
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
		frame.render_widget(no_response, area);
	}
}

fn draw_history_tab(frame: &mut Frame, area: Rect, app: &App) {
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
		frame.render_widget(no_history, area);
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

		frame.render_stateful_widget(history_list, area, &mut ratatui::widgets::ListState::default());
	}
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
	let should_hide_vim_mode = matches!(app.state, AppState::Normal | AppState::Help);

	let vim_mode_text = format!("-- {} --", app.vim.mode);
	let vim_mode_width = if should_hide_vim_mode {
		0
	} else {
		vim_mode_text.chars().count() as u16 + 2
	};

	let info_text = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
	let info_text_width = info_text.chars().count() as u16;

	let vim_mode_widget =
		Paragraph::new(vim_mode_text).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

	let mut keybindings_widget = if matches!(app.input_mode, InputMode::Editing) {
		Paragraph::new("Save: Enter | Cancel: Escape")
	} else {
		Paragraph::new("Help: ? | Switch tabs: Tab | Change method: m/M | Send request: Enter")
	}
	.style(Style::default().fg(Color::Yellow));

	if let Some(error) = &app.error_message {
		keybindings_widget = Paragraph::new(format!("Error: {error}")).style(Style::default().fg(Color::Red));
	}

	let info_widget = Paragraph::new(info_text).style(Style::default().fg(Color::Magenta));

	let layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Length(vim_mode_width),
			Constraint::Min(0),
			Constraint::Length(info_text_width),
		])
		.split(area);

	if should_hide_vim_mode {
		frame.render_widget(vim_mode_widget, layout[0]);
	}

	frame.render_widget(keybindings_widget, layout[1]);
	frame.render_widget(info_widget, layout[2]);
}

fn draw_help(frame: &mut Frame, area: Rect) {
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
	frame.render_widget(help_paragraph, area);
}

fn draw_loading_popup(frame: &mut Frame) {
	let popup_area = centered_rect(30, 10, frame.area());

	frame.render_widget(Clear, popup_area);

	let loading = Paragraph::new("Sending request...")
		.style(Style::default().fg(Color::Yellow))
		.alignment(Alignment::Center)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Loading")
				.border_style(Style::default().fg(Color::Yellow)),
		);
	frame.render_widget(loading, popup_area);
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
