#!/bin/bash

cp_rust_files() {
    rust_tool_dir="$HOME/untitled/rust/projects/hyprland_eww_tools"
    src_dir="$HOME/untitled/hyprdots/hyprland_eww_tools"
    cp -r "$src_dir/$1/src" "$src_dir/$1/Cargo.toml" "$rust_tool_dir/$1/"
}

cp_rust_files ipc_listener

cp_rust_files pipewire_update_on_change

cp_rust_files battery_fetcher

cp_rust_files playerctl_length_cut
