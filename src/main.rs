use anyhow::Result;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;
use vim::{Transition, Vim};

mod app;
mod http_client;
mod request;
mod response;
mod ui;
mod vim;

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

	if let Err(error) = res {
		println!("Error: {error}");
	}

	Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
	loop {
		terminal.draw(|frame| ui::draw(frame, app))?;

		let mut vim = Vim::new(Mode::Normal);

		if event::poll(Duration::from_millis(100))? {
			vim = match vim.transition(crossterm::event::read()?.into(), &mut textarea) {
				Transition::Mode(mode) if vim.mode != mode => {
					textarea.set_block(mode.block());
					textarea.set_cursor_style(mode.cursor_style());
					Vim::new(mode)
				}
				Transition::Nop | Transition::Mode(_) => vim,
				Transition::Pending(input) => vim.with_pending(input),
				Transition::Quit => return Ok(()),
			}

			// if let Event::Key(key) = event::read()? {
			// 	app.handle_key_event(key).await?;
			// }
		}

		app.update().await?;
	}
}
