#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    adb_powerhub_lib::run()
}