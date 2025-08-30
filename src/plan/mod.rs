#![allow(unused_imports)]
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Africa::Johannesburg;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;
#[derive(Serialize, Deserialize, Debug)]
pub struct Plan {
	pub title: String,
	pub description: String,
	// creation date in utc
	pub added_utc: String,
	// creation date in local time should query system time or locale
	pub added_local: String,
}
#[allow(dead_code)]
impl Plan {
	// Constructor for new plan object
	pub fn new(
		title: String,
		description: String,
	) -> Self {
		let (added_utc, added_local) = fetch_time();
		Plan {
			title,
			description,
			added_utc,
			added_local,
		}
	}
	pub fn get_title(&self) -> &str {
		&self.title
	}
	pub fn get_desc(&self) -> &str {
		&self.description
	}
	pub fn get_time_utc(&self) -> &str {
		&self.added_utc
	}
	pub fn parse(json_line: &str) -> Result<Plan, Error> {
		match serde_json::from_str::<Plan>(json_line) {
			Ok(plan) => Ok(plan),
			Err(e) => Err(e),
		}
	}
	pub fn formatted_dt(&self) -> Option<String> {
		let _dt = DateTime::parse_from_rfc3339(&self.added_local);
		match _dt {
			Ok(dt) => Some(dt.format("%d %b %Y %H:%M:%S").to_string()),
			Err(e) => {
				println!("Error formatting local_time: {}", e);
				None
			}
		}
	}
}
fn fetch_time() -> (String, String) {
	match sntp_time() {
		Some(dt_utc) => {
			let local = dt_utc.with_timezone(&Johannesburg);
			(dt_utc.to_rfc3339(), local.to_rfc3339())
		}
		None => {
			eprintln!("(NTP failed, using system clock)");
			let now = Utc::now();
			let local = now.with_timezone(&Johannesburg);
			(now.to_rfc3339(), local.to_rfc3339())
		}
	}
}
fn sntp_time() -> Option<chrono::DateTime<Utc>> {
	let addr = ("time.google.com", 123).to_socket_addrs().ok()?.next()?;
	let socket = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
	socket.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
	socket.set_write_timeout(Some(Duration::from_secs(2)))
		.ok()?;

	let mut packet = [0u8; 48];
	packet[0] = (0 << 6) | (4 << 3) | 3; // LI=0, VN=4, Mode=3
	socket.send_to(&packet, addr).ok()?;
	let mut buf = [0u8; 48];
	socket.recv_from(&mut buf).ok()?;

	const NTP_UNIX_EPOCH_DIFF: i64 = 2_208_988_800;
	let secs =
		u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]) as i64;
	let frac =
		u32::from_be_bytes([buf[44], buf[45], buf[46], buf[47]]) as u64;
	let nanos = ((frac as u128 * 1_000_000_000u128) >> 32) as u32;
	let unix_secs = secs.checked_sub(NTP_UNIX_EPOCH_DIFF)?;
	Utc.timestamp_opt(unix_secs, nanos).single()
}
