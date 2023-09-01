// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use std::sync::{Arc, Mutex};

use log::{debug, error};
use serde::Serialize;
use tauri::{api::dialog::blocking::FileDialogBuilder, State};

use crate::books::models::BookError;
use crate::books::{self, BookManager, BookPool};
use crate::rec_pois;
use crate::settings::{UserSettings, SettingsError};

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
    BookError::EmptyAuthors => from_err_api!(43)
);

from_err_api!(books::Error,
    books::Error::PoolAlreadyAdded => from_err_api!(20),
    books::Error::PoolNotFound => from_err_api!(21),
    books::Error::CurrentPoolNotSet => from_err_api!(22),
    books::Error::BookError(e) =>  e.into(),
    books::Error::ConversionFailed => from_err_api!(23)
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
pub async fn current_lang(settings: State<'_, UserSettingsAPI>) -> Result<String> {
    Ok(settings.get_current_lang())
}

/*******************************************************
 *
 * Book API
 *
 ******************************************************/

#[derive(Default)]
pub struct BookManagerState(Arc<Mutex<BookManager>>);

#[tauri::command]
pub async fn create_book_db(manager: State<'_, BookManagerState>, settings: State<'_, UserSettingsAPI>) -> Result<String> {
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

    Ok(key)
}
