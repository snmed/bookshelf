// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use std::sync::{Arc, Mutex};

use log::debug;
use serde::Serialize;
use tauri::{api::dialog::blocking::FileDialogBuilder, State};

use crate::books::models::BookError;
use crate::books::{self, BookManager, BookPool};
use crate::rec_pois;
use crate::settings::UserSettings;

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



/*******************************************************
 *
 * Book API
 *
 ******************************************************/

#[derive(Default)]
pub struct BookManagerState(Arc<Mutex<BookManager>>);

#[tauri::command]
pub async fn create_book_db(manager: State<'_, BookManagerState>) -> Result<String> {
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

    Ok(key)
}
