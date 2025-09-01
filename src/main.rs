#![allow(dead_code, unused_variables, unused_must_use)]
use color_eyre::eyre::Result;
use ratatui::layout::Alignment;
use ratatui::{
	DefaultTerminal, Frame,
	crossterm::{
		event::{self, Event},
		style::Color,
	},
	layout::{Constraint, Direction, Layout, Rect},
	style::{Style, Stylize},
	widgets::{
		Block, BorderType, Borders, List, ListItem, ListState, Paragraph,
		Widget, Wrap,
	},
};
#[derive(Debug)]
struct AppState {
	plans: Vec<Plan>,
	list_state: ListState,
	popup: Popup,
}
#[derive(Debug)]
enum Popup {
	None,
	ConfirmDelete(usize),
}

#[derive(Debug, Default)]
struct Plan {
	title: String,
	description: String,
	time_utc: String, // use utc and convert to local time when loaded
}

fn main() -> Result<()> {
	let mut state = AppState {
		plans: vec![],
		list_state: ListState::default(),
		popup: Popup::None,
	};
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
		//rendering
		terminal.draw(|f| render(f, app_state))?;
		//input_handling
		// blocking thread for some reason!
		match app_state.popup {
			Popup::ConfirmDelete(index) => {
				if let Event::Key(key) = event::read()? {
					match key.code {
						event::KeyCode::Char('y') => {
							app_state.plans.remove(index);
							app_state.popup = Popup::None;
						}
						event::KeyCode::Char('n')
						| event::KeyCode::Esc => app_state.popup = Popup::None,
						_ => {}
					}
				}
			}
			Popup::None => {
				if let Event::Key(key) = event::read()? {
					match key.code {
						event::KeyCode::Char('q') => break,
						event::KeyCode::Char(c) => {
							match c {
								'j' => app_state
									.list_state
									.select_next(),

								'k' => app_state
									.list_state
									.select_previous(),

								'D' => {
									if let Some(selected) =
										app_state
											.list_state
											.selected()
									{
										verify_deletion(
											selected,
											app_state,
										);
									}
								}
								// '' => {},
								// '' => {},
								// '' => {},
								// '' => {},
								// '' => {},
								// '' => {},
								// '' => {},
								_ => {}
							}
						}
						_ => {}
					}
				};
			}
		}
	}
	Ok(())
}

fn verify_deletion(index: usize, app_state: &mut AppState) {
	app_state.popup = Popup::ConfirmDelete(index)
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
	let list = List::new(
		app_state
			.plans
			.iter()
			.map(|i| ListItem::from(i.description.clone())),
	)
	.highlight_symbol(">")
	.highlight_style(Style::default().bg(Color::DarkGrey.into()));
	frame.render_stateful_widget(list, list_area, &mut app_state.list_state);

	if let Popup::ConfirmDelete(index) = app_state.popup {
		let popup_area = centered_rect(40, 8, frame.area());
		let block = Block::default()
			.title("Confirm Deletion")
			.borders(Borders::ALL)
			.border_type(BorderType::Thick)
			.style(Style::default().fg(Color::Red.into()));
		frame.render_widget(block, popup_area);

		let inner = Layout::default()
			.margin(1)
			.constraints([Constraint::Percentage(100)].as_ref())
			.split(popup_area)[0];

		let text = Paragraph::new(
			"Are you sure you want to delete this plan? (y/n)",
		)
		.alignment(Alignment::Center)
		.wrap(Wrap { trim: true });
		frame.render_widget(text, inner);
	}
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
	let popup_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints(
			[
				Constraint::Percentage((100 - percent_y) / 2),
				Constraint::Percentage(percent_y),
				Constraint::Percentage((100 - percent_y) / 2),
			]
			.as_ref(),
		)
		.split(r);
	Layout::default()
		.direction(Direction::Horizontal)
		.constraints(
			[
				Constraint::Percentage((100 - percent_x) / 2),
				Constraint::Percentage(percent_x),
				Constraint::Percentage((100 - percent_x) / 2),
			]
			.as_ref(),
		)
		.split(popup_layout[1])[1]
}
