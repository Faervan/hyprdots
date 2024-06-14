use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::Command;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize)]
struct Workspace {
    id: u8,
    name: String,
}

#[allow(dead_code)]
fn print_log(msg: &str) {
    Command::new("sh")
        .arg("-c")
        .arg(format!("notify-send \"{}\"", msg))
        .spawn().expect("log notify failed");
}

fn get_workspace_box(workspaces: &Vec<Workspace>, active: u8) -> String {
    let mut workspace_box = String::from("(box :orientation \"h\" :height \"30\" :class \"module workspaces\" :space-evenly false ");
    for workspace in workspaces.iter() {
        workspace_box.push_str("(eventbox :class \"workspace_event_box");
        if workspace.id == active {
            workspace_box.push_str(" active");
        }
        workspace_box.push_str(format!(" workspace_{}\" :width \"30\" \"{}\")", workspace.id, workspace.name).as_str());
    }
    workspace_box.push_str(")\n");
    workspace_box
}

fn main() -> io::Result<()> {
    let socket = format!(
        "{}/hypr/{}/.socket2.sock",
        env::var("XDG_RUNTIME_DIR").unwrap(),
        env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap()
    );
    let stream = UnixStream::connect(socket)?;
    let reader = BufReader::new(stream);
    let pipe = Path::new("/tmp/hypr_eww_ipc_listener_pipe");
    let mut writer = OpenOptions::new()
        .write(true)
        .open(&pipe)?;

    let mut workspaces: Vec<Workspace> = serde_json::from_str(String::from_utf8(Command::new("sh")
        .arg("-c")
        .arg("hyprctl -j workspaces")
        .output()
        .expect("failed to execute process").stdout).unwrap().as_str())?;
    let mut active_workspace: Workspace = serde_json::from_str(String::from_utf8(Command::new("sh")
        .arg("-c")
        .arg("hyprctl -j activeworkspace")
        .output()
        .expect("failed to execute process").stdout).unwrap().as_str())?;
    workspaces.sort_unstable_by_key(|w| w.id);
    let workspace_box = get_workspace_box(&workspaces, active_workspace.id);
    writer.write_all(&workspace_box.into_bytes())?;

    for line in reader.lines() {
        match line {
            Ok(event) => {
                if let Some(title) = event.as_str().strip_prefix("activewindow>>") {
                    let title: Vec<&str> = title.split(',').collect();
                    let mut title = title[1].to_string();
                    if title.len() > 30 {
                        title.truncate(27);
                        title.push_str("...");
                    }
                    println!("{}", title);
                } else if let Some(_) = event.as_str().strip_prefix("createworkspace>>") {
                    workspaces = serde_json::from_str(String::from_utf8(Command::new("sh")
                        .arg("-c")
                        .arg("hyprctl -j workspaces")
                        .output()
                        .expect("failed to execute process").stdout).unwrap().as_str())?;
                    workspaces.sort_unstable_by_key(|w| w.id);
                    let workspace_box = get_workspace_box(&workspaces, active_workspace.id);
                    writer.write_all(&workspace_box.into_bytes())?;
                } else if let Some(workspace) = event.as_str().strip_prefix("destroyworkspacev2>>") {
                    let workspace: Vec<&str> = workspace.split_terminator(",").collect();
                    let workspace = u8::from_str(workspace[0]).unwrap();
                    workspaces.retain(|w| w.id != workspace);
                    let workspace_box = get_workspace_box(&workspaces, active_workspace.id);
                    writer.write_all(&workspace_box.into_bytes())?;
                } else if let Some(workspace) = event.as_str().strip_prefix("workspacev2>>") {
                    let workspace: Vec<&str> = workspace.split_terminator(",").collect();
                    active_workspace.id = u8::from_str(workspace[0]).unwrap();
                    let workspace_box = get_workspace_box(&workspaces, active_workspace.id);
                    writer.write_all(&workspace_box.into_bytes())?;
                }
            },
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                break;
            }
        }
    }
    Ok(())
}