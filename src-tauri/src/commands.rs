// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use std::sync::{Arc, Mutex};

use log::{debug, error};
use serde::Serialize;
use tauri::{api::dialog::blocking::FileDialogBuilder, State};
use tauri::{AppHandle, Manager};

use crate::books::models::{self, Book, BookError, SearchConfig, StoreResult};
use crate::books::{self, BookManager, BookManagerEvent, BookPool, BOOK_MANAGER_EVENTS};
use crate::rec_pois;
use crate::settings::{SettingsError, UserSettings};

macro_rules! from_err_api {
    ($code:literal) => {
        ApiError {error: "".to_string(), code: $code }
    };

    ($error:expr, $code:expr) => {
        ApiError {error: $error, code: $code }
    };
    ($from:ty, $($enum:pat $(if $pred:expr)* => $result:expr),* ) => {
        impl From<$from> for ApiError {
                    fn from(value: $from) -> Self {
                    let s = format!("{:?}", &value);
                    let mut ae = match value {
                        $(
                            $enum $(if $pred)* => $result
                        ),*
                    };

                    if ae.error == "" {
                        ae.error = s;
                    }

                    ae
                }
        }
    };
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ApiError {
    error: String,
    code: i64,
}

from_err_api!(BookError,
    BookError::Generic(s) => from_err_api!(s, 40),
    BookError::NotFound => from_err_api!(41),
    BookError::DBError(e) => from_err_api!(e.to_string(),42),
    BookError::EmptyAuthors => from_err_api!(43),
    BookError::InvalidBook{ field: _, reason: _} => from_err_api!(44)
);

from_err_api!(books::Error,
    books::Error::PoolAlreadyAdded => from_err_api!(20),
    books::Error::PoolNotFound => from_err_api!(21),
    books::Error::CurrentPoolNotSet => from_err_api!(22),
    books::Error::BookError(e) =>  e.into(),
    books::Error::ConversionFailed => from_err_api!(23)
);

from_err_api!(tauri::Error,
    e => from_err_api!(format!("{:?}",e), 11)
);

from_err_api!(SettingsError ,
    e => from_err_api!(format!("{:?}",e), 30)
);

#[derive(Debug)]
pub enum CommandError {
    UserAborted,
}

from_err_api!(CommandError,
    CommandError::UserAborted => from_err_api!(1)
);

type Result<T = (), E = ApiError> = std::result::Result<T, E>;

/*******************************************************
 *
 * Settings API
 *
 ******************************************************/
pub struct UserSettingsAPI(pub Arc<Mutex<UserSettings>>);

impl Default for UserSettingsAPI {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(UserSettings::from_user_dir())))
    }
}

impl UserSettingsAPI {
    pub fn get_menu_expanded(&self) -> bool {
        let settings = rec_pois!(self.0);
        settings.menu_expanded
    }

    pub fn set_menu_expanded(&self, menu_expanded: bool) {
        let mut settings = rec_pois!(self.0);
        settings.menu_expanded = menu_expanded
    }

    pub fn get_current_lang(&self) -> String {
        let settings = rec_pois!(self.0);
        settings.lang.to_owned()
    }

    pub fn set_current_lang<T>(&self, lang: T)
    where
        T: AsRef<str>,
    {
        let mut settings = rec_pois!(self.0);
        settings.lang = lang.as_ref().to_owned()
    }

    pub fn get_theme(&self) -> String {
        let settings = rec_pois!(self.0);
        settings.theme.to_owned()
    }

    pub fn set_theme<T>(&self, theme: T)
    where
        T: AsRef<str>,
    {
        let mut settings = rec_pois!(self.0);
        settings.theme = theme.as_ref().to_owned()
    }

    pub fn add_history<T>(&self, path: T)
    where
        T: AsRef<str>,
    {
        if path.as_ref().is_empty() {
            return;
        }

        let mut settings = rec_pois!(self.0);
        let p = path.as_ref().to_string();
        if !settings.book_history.contains(&p) {
            settings.book_history.push(p)
        }
        settings.book_history.sort();
    }

    pub fn remove_history<T>(&self, path: T)
    where
        T: AsRef<str>,
    {
        let mut settings = rec_pois!(self.0);
        let p = path.as_ref().to_string();
        settings.book_history.retain(|h| *h != p);
    }

    pub fn get_history(&self) -> Vec<String> {
        let s = rec_pois!(self.0);
        s.book_history.clone()
    }

    pub fn save_settings(&self) -> Result<(), SettingsError> {
        let s = rec_pois!(self.0);
        match s.save_to_user_dir() {
            Ok(_) => {
                debug!("successfully saved user settings");
                Ok(())
            }
            Err(e) => {
                error!("failed to save user settings {:?}", e);
                Err(e)
            }
        }
    }
}

#[tauri::command]
pub async fn get_history(settings: State<'_, UserSettingsAPI>) -> Result<Vec<String>> {
    debug!("calling get_history command");
    Ok(settings.get_history())
}

#[tauri::command]
pub async fn remove_history(path: String, settings: State<'_, UserSettingsAPI>) -> Result {
    debug!("calling remove_history command");
    settings.remove_history(path);
    Ok(())
}

#[tauri::command]
pub async fn set_lang(lang: String, settings: State<'_, UserSettingsAPI>) -> Result {
    debug!("calling set_lang command");
    settings.set_current_lang(lang);
    settings.save_settings()?;
    Ok(())
}

#[tauri::command]
pub async fn current_lang(settings: State<'_, UserSettingsAPI>) -> Result<String> {
    debug!("calling current_lang command");
    Ok(settings.get_current_lang())
}

#[tauri::command]
pub async fn set_theme(theme: String, settings: State<'_, UserSettingsAPI>) -> Result {
    debug!("calling set_theme command");
    settings.set_theme(theme);
    settings.save_settings()?;
    Ok(())
}

#[tauri::command]
pub async fn current_theme(settings: State<'_, UserSettingsAPI>) -> Result<String> {
    debug!("calling current_theme command");
    Ok(settings.get_theme())
}

#[tauri::command]
pub async fn set_menu_expanded(expanded: bool, settings: State<'_, UserSettingsAPI>) -> Result {
    debug!("calling set_menu_expanded command");
    settings.set_menu_expanded(expanded);
    settings.save_settings()?;
    Ok(())
}

#[tauri::command]
pub async fn get_menu_expanded(settings: State<'_, UserSettingsAPI>) -> Result<bool> {
    debug!("calling current_theme command");
    Ok(settings.get_menu_expanded())
}

/*******************************************************
 *
 * Book API
 *
 ******************************************************/

#[derive(Default)]
pub struct BookManagerState(Arc<Mutex<BookManager>>);

#[tauri::command]
pub async fn set_current_db(
    db: String,
    manager: State<'_, BookManagerState>,
    app: AppHandle,
) -> Result {
    debug!("calling set_current_db command with param: {}", db);
    let mut m = rec_pois!(manager.0);
    m.set_current_pool(&db)?;

    app.emit_all(BOOK_MANAGER_EVENTS, BookManagerEvent::CurrentDBChanged(db))?;

    Ok(())
}

#[tauri::command]
pub async fn fetch_book(
    search: SearchConfig<models::ConfigInitialized>,
    manager: State<'_, BookManagerState>,
) -> Result<StoreResult<Book>> {
    debug!("calling fetch_book command with params: {:?}", search);
    let m = rec_pois!(manager.0);
    let result = m.get_current_pool()?.fetch_books(search)?;
    Ok(result)
}

#[tauri::command]
pub async fn update_book(mut book: Book, manager: State<'_, BookManagerState>) -> Result<Book> {
    debug!("calling update_book command with book: {:?}", book);
    let m = rec_pois!(manager.0);
    m.get_current_pool()?.update_book(&mut book)?;
    Ok(book)
}

#[tauri::command]
pub async fn delete_book(id: i64, manager: State<'_, BookManagerState>) -> Result {
    debug!("calling delete_book command with id: {:?}", id);
    let m = rec_pois!(manager.0);
    m.get_current_pool()?.delete_book_by_id(id)?;
    Ok(())
}

#[tauri::command]
pub async fn add_book(mut book: Book, manager: State<'_, BookManagerState>) -> Result<i64> {
    debug!("calling add_book command with book: {:?}", book);
    let m = rec_pois!(manager.0);
    m.get_current_pool()?.add_book(&mut book)?;
    Ok(book.id)
}

#[tauri::command]
pub async fn get_book(id: i64, manager: State<'_, BookManagerState>) -> Result<Book> {
    debug!("calling get_book command with id: {}", id);
    let m = rec_pois!(manager.0);
    Ok(m.get_current_pool()?.get_book(id)?)
}

#[tauri::command]
pub async fn close_db(manager: State<'_, BookManagerState>, app: AppHandle) -> Result {
    debug!("calling close_db command");
    let mut m = rec_pois!(manager.0);
    let current = m.current_pool_name()?;

    m.remove_pool(current.clone());

    let db = m.get_pools().first().unwrap_or(&"").to_string();
    m.set_current_pool(&db)?;

    app.emit_all(BOOK_MANAGER_EVENTS, BookManagerEvent::CurrentDBChanged(db))?;
    app.emit_all(
        BOOK_MANAGER_EVENTS,
        BookManagerEvent::OpenDBChanged(m.get_pools().iter().map(|s| s.to_string()).collect()),
    )?;

    Ok(())
}

#[tauri::command]
pub async fn create_book_db(
    manager: State<'_, BookManagerState>,
    settings: State<'_, UserSettingsAPI>,
    app: AppHandle,
) -> Result<String> {
    debug!("calling create_book_db command");

    let mut path = FileDialogBuilder::new()
        .add_filter("DB", &[".db"])
        .save_file()
        .ok_or(CommandError::UserAborted)?;

    let pool = BookPool::new_sqlite_pool(&path)?;

    if let Some(e) = path.extension() {
        if e.to_ascii_lowercase() != "db" {
            path.set_extension("db");
        }
    } else {
        path.set_extension("db");
    }

    let key: String = path
        .file_name()
        .expect("Invalid file path, should never happen.")
        .to_string_lossy()
        .into();

    let mut mgr = rec_pois!(manager.0);
    mgr.add_pool(&key, pool)?;

    settings.add_history(path.to_str().unwrap_or_default());

    app.emit_all(
        BOOK_MANAGER_EVENTS,
        BookManagerEvent::OpenDBChanged(mgr.get_pools().iter().map(|s| s.to_string()).collect()),
    )?;

    Ok(key)
}
