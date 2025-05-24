use anyhow::Result;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;

mod app;
mod http_client;
mod request;
mod response;
mod ui;

use app::{App, AppState};

#[tokio::main]
async fn main() -> Result<()> {
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut app = App::new();
	let res = run_app(&mut terminal, &mut app).await;

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	if let Err(err) = res {
		println!("Error: {:?}", err);
	}

	Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
	loop {
		terminal.draw(|f| ui::draw(f, app))?;

		// Handle events with timeout to allow for background tasks
		if event::poll(Duration::from_millis(100))? {
			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					match key.code {
						KeyCode::Char('q') => {
							if app.state == AppState::Normal {
								return Ok(());
							}
						}
						KeyCode::Esc => {
							app.state = AppState::Normal;
							app.input_mode = app::InputMode::Normal;
						}
						_ => {
							app.handle_key_event(key).await?;
						}
					}
				}
			}
		}

		app.update().await?;
	}
}
