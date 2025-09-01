#![allow(dead_code, unused_variables, unused_must_use)]
use color_eyre::eyre::Result;
use ratatui::{
	DefaultTerminal, Frame,
	crossterm::{
		event::{self, Event},
		style::Color,
	},
	layout::{Constraint, Layout},
	style::Stylize,
	widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
};
#[derive(Debug, Default)]
struct AppState {
	plans: Vec<Plan>,
}

#[derive(Debug, Default)]
struct Plan {
	title: String,
	description: String,
	time_utc: String, // use utc and convert to local time when loaded
}

fn main() -> Result<()> {
	let mut state = AppState::default();
	for i in 0..10 {
		state.plans.push(Plan {
			title: "Test Item 1".to_string(),
			description: "This is an item for testing".to_string(),
			time_utc: "Time will go here eventually".to_string(),
		});
	}

	color_eyre::install()?;

	let terminal = ratatui::init();
	let result = run(terminal, &mut state);

	ratatui::restore();
	result
}
fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
	loop {
		terminal.draw(|f| render(f, app_state))?;
		//input_handling
		// blocking thread for some reason!
		if let Event::Key(key) = event::read()? {
			match key.code {
				event::KeyCode::Char('q') => break,
				_ => {}
			}
		};
		//rendering
	}
	Ok(())
}


fn foo() {
	bar();
}
fn bar() {
	()
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
	let [border_area] = Layout::vertical([Constraint::Fill(1)])
		.margin(1)
		.areas(frame.area());
	Block::bordered()
		.border_type(BorderType::Rounded)
		.fg(Color::Blue)
		.render(border_area, frame.buffer_mut());

	let [list_area] = Layout::vertical([Constraint::Fill(1)])
		.margin(1)
		.areas(border_area);
	List::new(
		app_state
			.plans
			.iter()
			.map(|i| ListItem::from(i.description.clone())),
	)
	.render(list_area, frame.buffer_mut());
}
