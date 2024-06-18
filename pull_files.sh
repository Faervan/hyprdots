#!/bin/bash

cp_rust_files() {
    rust_tool_dir="$HOME/untitled/rust/projects/hyprland_eww_tools/"
    cp -r "$rust_tool_dir/$1/src" "$rust_tool_dir/$1/Cargo.toml" "$HOME/untitled/hyprdots/hyprland_eww_tools/$1/"
}

cp_rust_files ipc_listener

cp_rust_files pipewire_update_on_change

cp_rust_files battery_fetcher

cp_rust_files playerctl_length_cut
