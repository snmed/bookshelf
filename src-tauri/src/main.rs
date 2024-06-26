// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::OsString, fs::File, path::PathBuf};

use commands::{BookManagerState, UserSettingsAPI};
use log::{info, LevelFilter};
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
};
use tauri::State;

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

#[tauri::command]
fn shutdown(app_handle: tauri::AppHandle, settings: State<'_, UserSettingsAPI>) {
    info!("shutting down application");
    let _ = settings.save_settings();
    app_handle.exit(0)
}

fn main() {
    setup_logging();

    info!("starting bookshelf application");

    tauri::Builder::default()
        .manage(BookManagerState::default())
        .manage(UserSettingsAPI::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            shutdown,
            commands::create_book_db,
            commands::current_lang,
            commands::set_lang,
            commands::remove_history,
            commands::get_history,
            commands::set_current_db,
            commands::close_db,
            commands::get_book,
            commands::add_book,
            commands::delete_book,
            commands::update_book,
            commands::fetch_book,
            commands::set_theme,
            commands::current_theme,
            commands::get_menu_expanded,
            commands::set_menu_expanded
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_logging() {
    #[cfg(not(debug_assertions))]
    let default_lvl: OsString = "Off".into();
    #[cfg(debug_assertions)]
    let default_lvl: OsString = "Debug".into();

    let log_lvl = std::env::var_os("BOOKSHELF_LOG").unwrap_or(default_lvl);
    let log_no_term = std::env::var_os("BOOKSHELF_LOG_NOTERM").unwrap_or("".into());
    let log_file: PathBuf = std::env::var_os("BOOKSHELF_LOG_FILE")
        .unwrap_or("".into())
        .into();

    let lvl = match log_lvl.to_str() {
        Some(l) => match l.to_ascii_lowercase().as_str() {
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Off,
        },
        None => LevelFilter::Off,
    };

    if lvl != LevelFilter::Off {
        let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
        if log_no_term.is_empty() {
            loggers.push(TermLogger::new(
                lvl,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ));
        }

        if !log_file.as_os_str().is_empty() {
            loggers.push(WriteLogger::new(
                lvl,
                ConfigBuilder::new().set_time_format_rfc3339().build(),
                File::options()
                    .append(true)
                    .create(true)
                    .open(log_file)
                    .expect("Failed to create log file"),
            ));
        }

        if !loggers.is_empty() {
            CombinedLogger::init(loggers).expect("Failed to initalize loggers");
        }
    }
}
