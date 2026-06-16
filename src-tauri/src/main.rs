#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    keyboard_locker_lib::run()
}
