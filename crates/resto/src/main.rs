use ratatui::{
	Terminal,
	backend::CrosstermBackend,
	crossterm::{
		event::{DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event, poll, read},
		execute,
		terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
	},
};
use std::io;
use std::time::Duration;

mod app;
mod curl;
mod http_client;
mod logger;
mod request;
mod response;
mod ui;
mod utils;

use app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	logger::initialize_logging()?;
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut app = App::new();
	let res = run_app(&mut terminal, &mut app);

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	if let Err(error) = res {
		println!("Error: {error}");
	}

	Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> anyhow::Result<()> {
	loop {
		terminal.draw(|frame| ui::draw(frame, app))?;

		if poll(Duration::from_millis(50))? {
			if let Ok(event) = read() {
				match event {
					Event::Key(key) => {
						let should_quit = app.handle_key_event(key)?;
						if should_quit {
							return Ok(());
						}
					},
					Event::Paste(text) => {
						app.handle_paste(&text)?;
					},
					_ => {},
				}
			}
		}

		app.update();
	}
}
