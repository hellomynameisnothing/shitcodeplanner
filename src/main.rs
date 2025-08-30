use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};

use crate::plan::Plan;

pub mod plan;

const PATH: &str = "plans.jsonl";
fn main() {
	let file = match std::fs::File::open(PATH) {
		Ok(f) => f,
		Err(e) => panic!("Error: {}", e),
	};

	println!("=== Existing plans ===");
	let reader = BufReader::new(file);
	for (i, line) in reader.lines().enumerate() {
		let line = match line {
			Ok(l) => l,
			Err(e) => panic!("Error: {}", e),
		};
		let plan = match Plan::parse(&line) {
			Ok(p) => p,
			Err(e) => panic!("Error: {}", e),
		};
		let dt = match plan.formatted_dt() {
			Some(t_dt) => t_dt,
			None => panic!("Invalid DateTime for Plan[{:?}]", plan),
		};
		println!(
			"[{}] {} - {}(local: {})",
			i + 1,
			plan.title,
			plan.description,
			dt
		);
	}
	println!("======================");
	println!(" "); // empty line

	let title = match input("Title for new Plan:") {
		Some(t) => t,
		None => panic!("Please Input a title!"),
	};
	let desc = match input("Enter Description: "){
		Some(d) => d,
		None => "".to_string(), // empty description
	};

	let p = plan::Plan::new(title, desc);

	let mut file = OpenOptions::new()
		.create(true)
		.append(true)
		.open(PATH)
		.unwrap();

	writeln!(file, "{}", serde_json::to_string(&p).unwrap()).unwrap();

	println!("✅ Plan saved.");
}
fn input(prompt: &str) -> Option<String> {
	let mut input = String::new();
	println!("{}", prompt);
	io::stdout().flush().expect("Failed to flush stdout");
	io::stdin()
		.read_line(&mut input)
		.expect("Failed to get input");
	Some(input.trim().to_string())
}
