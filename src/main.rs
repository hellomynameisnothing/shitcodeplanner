use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Africa::Johannesburg;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write, BufRead, BufReader};
use std::net::{UdpSocket, ToSocketAddrs};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Plan {
    title: String,
    description: String,
    added_utc: String,
    added_local: String,
}

fn main() {
    let store_path = "plans.jsonl";

    // Show existing plans
    if let Ok(file) = std::fs::File::open(store_path) {
        println!("=== Existing plans ===");
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            if let Ok(l) = line {
                if let Ok(p) = serde_json::from_str::<Plan>(&l) {
                    // Parse added_local to reformat it
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&p.added_local) {
                        let formatted_local = dt.format("%d %b %Y %H:%M:%S").to_string();
                        println!(
                            "[{}] {} - {} (local: {})",
                            i + 1,
                            p.title,
                            p.description,
                            formatted_local
                        );
                    } else {
                        println!(
                            "[{}] {} - {} (local: {})",
                            i + 1,
                            p.title,
                            p.description,
                            p.added_local
                        );
                    }
                }
            }
        }
        println!("======================\n");
    }

    // Ask user for a new plan
    let mut title = String::new();
    print!("Enter new plan title (empty to quit): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut title).unwrap();
    let title = title.trim().to_string();
    if title.is_empty() {
        return;
    }

    let mut desc = String::new();
    print!("Enter description: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut desc).unwrap();
    let desc = desc.trim().to_string();

    let (utc_iso, local_iso) = fetch_time();

    let plan = Plan {
        title,
        description: desc,
        added_utc: utc_iso,
        added_local: local_iso,
    };

    // Append to file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(store_path)
        .unwrap();
    writeln!(file, "{}", serde_json::to_string(&plan).unwrap()).unwrap();

    println!("✅ Plan saved.");
}

/// Query time.google.com (SNTP). Fallback to system clock.
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

/// Minimal SNTP client
fn sntp_time() -> Option<chrono::DateTime<Utc>> {
    let addr = ("time.google.com", 123).to_socket_addrs().ok()?.next()?;
    let socket = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    socket.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    socket.set_write_timeout(Some(Duration::from_secs(2))).ok()?;

    let mut packet = [0u8; 48];
    packet[0] = (0 << 6) | (4 << 3) | 3; // LI=0, VN=4, Mode=3
    socket.send_to(&packet, addr).ok()?;
    let mut buf = [0u8; 48];
    socket.recv_from(&mut buf).ok()?;

    const NTP_UNIX_EPOCH_DIFF: i64 = 2_208_988_800;
    let secs = u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]) as i64;
    let frac = u32::from_be_bytes([buf[44], buf[45], buf[46], buf[47]]) as u64;
    let nanos = ((frac as u128 * 1_000_000_000u128) >> 32) as u32;
    let unix_secs = secs.checked_sub(NTP_UNIX_EPOCH_DIFF)?;
    Utc.timestamp_opt(unix_secs, nanos).single()
}
