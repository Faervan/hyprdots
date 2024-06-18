use std::{io::{BufRead, BufReader}, process::{Command, Stdio}};

const MAX_LENGTH: usize = 35;
const MAX_TITLE_LENGTH: usize = 20;

fn main() {
    let stdout = Command::new("sh")
        .arg("-c")
        .arg("playerctl --follow metadata --format '{{title}}¾{{artist}}'")
        .stdout(Stdio::piped())
        .spawn().unwrap()
        .stdout
        .expect("Failed to read output from playerctl");

    let reader = BufReader::new(stdout);
    print!("{}",
        String::from_utf8(Command::new("sh")
        .arg("-c")
        .arg("playerctl metadata | grep \"title\\|artist\" | cut -c 37-")
        .output()
        .expect("failed to execute process").stdout).unwrap()
    );
    for line in reader.lines().filter_map(|line| line.ok()) {
        if let Some((mut title, artist)) = line.split_once('¾') {
            title = title.trim();
            if title != "" {
                let mut title = title.to_string();
                let mut artist = artist.to_string();
                if title.len() + artist.len() + 3 > MAX_LENGTH {
                    if title.len() > MAX_TITLE_LENGTH {
                        title.truncate(MAX_TITLE_LENGTH - 3);
                        title.push_str("...");
                    }
                    if title.len() + artist.len() + 3 > MAX_LENGTH {
                        artist.truncate(MAX_LENGTH - MAX_TITLE_LENGTH - 6);
                        artist.push_str("...");
                    }
                }
                if artist != "".to_string() {
                    println!("{title} - {artist}");
                } else {
                    println!("{title}x");
                }
            }
        }
    }
}