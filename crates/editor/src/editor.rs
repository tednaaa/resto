use rat_text::text_area::{TextArea, TextAreaState};
use ratatui::{
	buffer::Buffer,
	layout::Rect,
	style::{Style, Stylize},
	widgets::StatefulWidget,
};

pub struct EditorState {
	textarea: TextAreaState,
}

pub struct Editor;

impl Editor {
	pub const fn new() -> Self {
		Self {}
	}
}

impl StatefulWidget for Editor {
	type State = EditorState;

	fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
		TextArea::new()
			.set_horizontal_max_offset(256)
			.style(Style::default().white().on_dark_gray())
			.select_style(Style::default().black().on_yellow())
			.text_style([Style::new().red(), Style::new().underlined(), Style::new().green(), Style::new().on_yellow()])
			.render(area, buffer, &mut state.textarea);
	}
}
