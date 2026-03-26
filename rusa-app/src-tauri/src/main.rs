// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bcrypt::{hash, DEFAULT_COST};

fn main() {
    let password = "admin123";
    let hashed = hash(password, DEFAULT_COST).unwrap();
    println!("HASH: {}", hashed);

    rusa_app_lib::run();
}