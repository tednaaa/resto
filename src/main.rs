use anyhow::Result;
use ratatui::{
	Terminal, 
	backend::CrosstermBackend,
	crossterm::{
		event::{self, DisableMouseCapture, EnableMouseCapture},
		execute,
		terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
	},
};
use std::io;
use std::time::Duration;


mod app;
mod http_client;
mod request;
mod response;
mod ui;
mod vim;

use app::App;

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

	if let Err(error) = res {
		println!("Error: {error}");
	}

	Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
	loop {
		terminal.draw(|frame| ui::draw(frame, app))?;

		if event::poll(Duration::from_millis(100))? {
			if let ratatui::crossterm::event::Event::Key(key) = event::read()? {
				let should_quit = app.handle_key_event(key).await?;
				if should_quit {
					return Ok(());
				}
			}
		}

		app.update().await?;
	}
}
