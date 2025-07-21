use gpui::{App, Application, WindowOptions, prelude::*};

mod views;

fn main() {
	Application::new().run(|cx: &mut App| {
		vim::init(cx);
		cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| views::root::RootView)).unwrap();
	});
}
