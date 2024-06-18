use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;
use async_std::task;
use udev::{self, Device};

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
        if event.sysname() == "AC" {
            thread::sleep(Duration::from_secs(1));
            print_battery_stats(get_charging_state());
        }
    }
}

fn read_properties<'a>(bat: &str, properties: &Vec<&'a str>) -> std::io::Result<HashMap<&'a str, String>> {
    let bat = Device::from_syspath(Path::new(format!("/sys/class/power_supply/{}", bat).as_str()))?;
    let mut map: HashMap<&str, String> = HashMap::new();
    for prop in properties.iter() {
        map.insert(prop, bat.property_value(format!("POWER_SUPPLY_{}", prop.to_uppercase())).unwrap().to_str().unwrap().to_string());
    }
    Ok(map)
}

fn get_charging_state() -> bool {
    let mut is_charging = false;
    if let Ok(bat0) = Device::from_syspath(Path::new("/sys/class/power_supply/BAT0")) {
        if bat0.property_value("POWER_SUPPLY_STATUS").unwrap().to_str().unwrap() == "Charging" {
            is_charging = true;
        }
    }
    if let Ok(bat1) = Device::from_syspath(Path::new("/sys/class/power_supply/BAT1")) {
        if bat1.property_value("POWER_SUPPLY_STATUS").unwrap().to_str().unwrap() == "Charging" {
            is_charging = true;
        }
    }
    is_charging
}

fn print_battery_stats(is_charging: bool) {
    let (mut total_energy, mut total_energy_full, mut power_now, mut voltage_now) = (0., 0., 0., 0.);
    let mut batteries: Vec<HashMap<&str, String>> = Vec::new();
    let properties = vec!["energy_now", "energy_full", "power_now", "voltage_now"];
    loop {
        if let Ok(bat) = read_properties("BAT0", &properties) {
            batteries.push(bat);
        }
        if let Ok(bat) = read_properties("BAT1", &properties) {
            batteries.push(bat);
        }
        let mut is_valid = false;
        for bat in batteries.iter() {
            if bat.get("power_now").unwrap().parse::<f32>().unwrap_or(0.) != 0. {
                is_valid = true;
            }
        }
        if is_valid {break;}
        thread::sleep(Duration::from_secs(1));
    }
    for bat in batteries.iter() {
        total_energy += bat.get("energy_now").unwrap().parse::<f32>().unwrap_or(0.);
        total_energy_full += bat.get("energy_full").unwrap().parse::<f32>().unwrap_or(0.);
        let bat_power_now = bat.get("power_now").unwrap().parse::<f32>().unwrap_or(0.);
        if power_now == 0. && bat_power_now != 0. {
            power_now = bat_power_now;
            voltage_now = bat.get("voltage_now").unwrap().parse::<f32>().unwrap_or(0.);
        }
    }
    let total_capacity = (100. / total_energy_full * total_energy).round() as usize;
    let icon = match is_charging {
        false => match total_capacity {
            n if n >= 80 => "󱊣",
            n if n >= 60 => "󱊢",
            n if n >= 25 => "󱊡",
            _ => "󰂎"
        },
        true => match total_capacity {
            n if n >= 80 => "󱊦 ",
            n if n >= 60 => "󱊥 ",
            n if n >= 25 => "󱊤 ",
            _ => "󰢟"
        }
    };
    println!("{{ \"total_capacity\": {}, {} \"icon\": \"{}\" }}",
        total_capacity,
        match is_charging {
            false => {
                let time_remaining = total_energy / power_now;
                format!("\"time_remaining\": {}, \"time_remaining_hours\": {}, \"time_remaining_mins\": {},",
                    (time_remaining * 100.).round() / 100.,
                    time_remaining.floor(),
                    ((time_remaining - time_remaining.floor()) *60.) as u32,
                )
            },
            true => {
                //The 12.55 multiplicator makes absolutely zero sense, but by doing that we can archieve an output similar to upower
                //This still doesn't match up with the actual time at all ... lol
                let time_remaining = (total_energy_full - total_energy) / 1000000. / (( power_now / 1000000. ) * ( voltage_now / 1000000. )) * 12.55;
                format!("\"time_remaining\": {}, \"time_remaining_hours\": {}, \"time_remaining_mins\": {},",
                    (time_remaining * 100.).round() / 100.,
                    time_remaining.floor(),
                    ((time_remaining - time_remaining.floor()) *60.) as u32,
                )
            },
        },
        icon,
    );
}

/*
    How upower calculates remaining time (upower/src/up-daemon.c):
    /* calculate a quick and dirty time remaining value
	 * NOTE: Keep in sync with per-battery estimation code! */
	if (energy_rate_total > 0) {
		if (state_total == UP_DEVICE_STATE_DISCHARGING)
			time_to_empty_total = SECONDS_PER_HOUR * (energy_total / energy_rate_total);
		else if (state_total == UP_DEVICE_STATE_CHARGING)
			time_to_full_total = SECONDS_PER_HOUR * ((energy_full_total - energy_total) / energy_rate_total);
	}
*/