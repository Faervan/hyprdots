use std::{io::{BufRead, BufReader}, process::{Command, Stdio}};

fn get_volume() -> String {
    let volume = String::from_utf8(
        Command::new("sh")
            .arg("-c")
            .arg("wpctl get-volume @DEFAULT_AUDIO_SINK@")
            .stdout(Stdio::piped())
            .output()
            .expect("Couldn't get volume from wpctl")
            .stdout
        ).expect("Couldn't convert output from wpctl to String");
    let mut volume = volume.as_str().strip_prefix("Volume: ").unwrap().trim();
    let mut muted = false;
    if let Some(x) = volume.strip_suffix(" [MUTED]") {
        volume = x;
        muted = true;
    }
    let volume = volume.parse::<f32>().unwrap();
    let volume = (volume * 100.) as u16;
    format!("{{ \"perc\": {}, \"class\": \"{}\", \"icon\": \"{}\" }}",
        volume,
        if muted || volume >= 50 {
            "icon extra-margin-right"
        } else {
            "icon bigger-icon-font"
        },
        if muted {
            "󰖁"
        } else {
            match volume {
                n if n >= 50 => "󰕾",
                n if n >= 25 => "󰖀",
                _ => "󰕿"
            }
        }
    )
}

fn main() {
    println!("{}", get_volume());
    let stdout = Command::new("sh")
        .arg("-c")
        .arg("pactl subscribe")
        .stdout(Stdio::piped())
        .spawn().unwrap()
        .stdout
        .expect("Failed to read output from playerctl");

    let reader = BufReader::new(stdout);
    for line in reader.lines().filter_map(|line| line.ok()) {
        if line.as_str().contains("change") {
            println!("{}", get_volume());
        }
    }
}