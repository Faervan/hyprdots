#!/bin/bash

hyprdots_dir="$HOME/untitled/hyprdots"

cp_rust_files() {
    rust_tool_dir="$HOME/untitled/rust/projects/hyprland_eww_tools/"
    cp -r "$rust_tool_dir/$1/src" "$rust_tool_dir/$1/Cargo.toml" "$hyprdots_dir/hyprland_eww_tools/$1/"
}

cp_config_folder() {
    config_dir="$HOME/.config/$1"
    cp -r "$config_dir" "$hyprdots_dir/config"
}

cp_rust_files ipc_listener

cp_rust_files pipewire_update_on_change

cp_rust_files battery_fetcher

cp_rust_files playerctl_length_cut

cp_config_folder eww
