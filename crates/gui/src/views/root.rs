use gpui::{Context, Window, div, prelude::*, px, rgb};
use ui_input::SingleLineInput;
use vim;

pub struct RootView;

impl Render for RootView {
	fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		vim::init(cx);
		// let url_input = cx.new(|cx| SingleLineInput::new(window, cx, "Write URL or paste curl...").label("URL"));

		// let url = url_input.read(cx).editor().read(cx).text(cx).trim().to_string();

		div().bg(rgb(0x505050)).size_full().flex().flex_col().gap_3().child(div().child(div())).child(div())
	}
}
