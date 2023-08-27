// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{sync::{Arc, Mutex}, fs::File};

use books::{models::Book, BookManager};
use commands::BookManagerState;
use log::{debug, error, trace, info, warn, LevelFilter};
use simplelog::{CombinedLogger, TermLogger, WriteLogger, Config, TerminalMode, ColorChoice};
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

    // Todo: Move initialization into own function and make it configurable with env vars.
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("my_rust_bin.log").unwrap())
        ]
    ).expect("Failed to initialize logger");
 
    trace!("a trace example");
    debug!("deboogging");
    info!("such information");
    warn!("o_O");
    error!("boom");

    tauri::Builder::default()
        .manage(BookManagerState::default())
        .invoke_handler(tauri::generate_handler![greet, create_book_db])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
