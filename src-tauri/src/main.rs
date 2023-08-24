// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use books::{models::Book, BookManager};
use commands::BookManagerState;
use tauri::State;

use crate::commands::create_book_db;

// Module declarations
mod books;
mod commands;
mod macros;
mod pool;
mod settings;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .manage(BookManagerState::default())
        .invoke_handler(tauri::generate_handler![greet, create_book_db])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
