use std::fs::File;
use std::io::{self, prelude::*};
use std::thread;
use std::time::Duration;
use async_std::task;
use udev::{self};

fn main() -> io::Result<()> {
    print_battery_stats(get_charging_state());
    task::spawn(async {
        loop {
            task::sleep(Duration::from_secs(30)).await;
            print_battery_stats(get_charging_state());
        }
    });
    let socket = udev::MonitorBuilder::new()?
        .match_subsystem_devtype("power_supply", "power_supply")?
        .listen()?;
    loop {
        let event = match socket.iter().next() {
            Some(event) => event,
            None => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
        };
        print_event(event);
    }
}

fn print_event(event: udev::Event) {
    println!(
        "{}: {} {} (subsystem={}, sysname={}, devtype={})",
        event.sequence_number(),
        event.event_type(),
        event.syspath().to_str().unwrap_or("---"),
        event
            .subsystem()
            .map_or("", |s| { s.to_str().unwrap_or("") }),
        event.sysname().to_str().unwrap_or(""),
        event.devtype().map_or("", |s| { s.to_str().unwrap_or("") })
    );
}

fn read_file(bat: &str, file: &str) -> std::io::Result<String> {
    let mut file = File::open(format!("/sys/class/power_supply/{}/{}", bat, file).as_str())?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content.trim().to_string())
}

fn get_charging_state() -> bool {
    let mut is_charging = false;
    if let Ok(bat0_charging) = read_file("BAT0", "status") {
        if bat0_charging == "Charging".to_string() {is_charging = true;}
    }
    if let Ok(bat1_charging) = read_file("BAT1", "status") {
        if bat1_charging == "Charging".to_string() {is_charging = true;}
    }
    is_charging
}

fn print_battery_stats(is_charging: bool) {
    let mut total_perc: usize = 0;
    let mut total_energy: f32 = 0.;
    let mut total_power_now: f32 = 0.;
    let mut batteries: Vec<String> = Vec::new();
    if let Ok(bat0_perc) = read_file("BAT0", "capacity") {
        let bat0_perc = bat0_perc.parse::<usize>().unwrap();
        batteries.push(format!("\"BAT0_perc\": {bat0_perc}"));
        total_perc += bat0_perc;
    }
    if let Ok(bat1_perc) = read_file("BAT1", "capacity") {
        let bat1_perc = bat1_perc.parse::<usize>().unwrap();
        batteries.push(format!("\"BAT1_perc\": {bat1_perc}"));
        total_perc += bat1_perc;
    }
    if let (Ok(bat0_power_now), Ok(bat0_energy_now)) = (read_file("BAT0", "power_now"), read_file("BAT0", "energy_now")) {
        let bat0_power_now = bat0_power_now.parse::<f32>().unwrap();
        let bat0_energy_now = bat0_energy_now.parse::<f32>().unwrap();
        total_energy += bat0_energy_now;
        total_power_now += bat0_power_now;
    }
    if let (Ok(bat1_power_now), Ok(bat1_energy_now)) = (read_file("BAT1", "power_now"), read_file("BAT1", "energy_now")) {
        let bat1_power_now = bat1_power_now.parse::<f32>().unwrap();
        let bat1_energy_now = bat1_energy_now.parse::<f32>().unwrap();
        total_energy += bat1_energy_now;
        total_power_now += bat1_power_now;
    }
    let mut result = "{ ".to_string();
    for battery in batteries.iter() {
        result.push_str(format!("{battery}, ").as_str());
    }
    let icon = match is_charging {
        false => match total_perc / batteries.len() {
            n if n >= 80 => "󱊣",
            n if n >= 60 => "󱊢",
            n if n >= 25 => "󱊡",
            _ => "󰂎"
        },
        true => match total_perc / batteries.len() {
            n if n >= 80 => "󱊦",
            n if n >= 60 => "󱊥",
            n if n >= 25 => "󱊤",
            _ => "󰢟"
        }
    };
    //We can get the remaining time by deviding the total energy by the total "power_now" (the power consumption right now)
    //Before we do that, we convert from microwatt to watt / from microwatt hours to watt hours
    let time_remaining = (total_energy / 1000000.) / (total_power_now / 1000000.);
    result.push_str(format!("\"total_perc\": {}, \"time_remaining\": {}, \"time_remaining_hours\": {}, \"time_remaining_mins\": {}, \"icon\": \"{}\" }}",
        total_perc / batteries.len(),
        (time_remaining*100.).round()/100.,
        time_remaining.floor(),
        ((time_remaining - time_remaining.floor()) *60.) as u32,
        icon
    ).as_str());
    //Print in JSON Format
    println!("{result}");
}