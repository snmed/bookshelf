// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use directories::UserDirs;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    result,
};

use crate::from_err;

#[cfg(not(windows))]
const SETTINGS_FILE: &str = ".config/bookshelf/bookshelf-settings.json";
#[cfg(windows)]
const SETTINGS_FILE: &str = r"bookshelf\bookshelf-settings.json";

#[inline]
fn get_user_settings_path() -> Result<PathBuf> {
    Ok(UserDirs::new()
        .ok_or(SettingsError::UserDirNotFound)?
        .home_dir()
        .join(SETTINGS_FILE))
}

/// Errors which can results within this module.
#[derive(Debug)]
pub enum SettingsError {
    UserDirNotFound,
    InvalidPath,
    SerdeError(serde_json::Error),
    IoError(io::Error),
}

from_err!(SettingsError, serde_json::Error, SerdeError);
from_err!(SettingsError, io::Error, IoError);

pub type Result<T = (), E = SettingsError> = result::Result<T, E>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct UserSettings {
    pub lang: String,
    pub book_history: Vec<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            lang: "en".to_owned(),
            book_history: Default::default(),
        }
    }
}

impl UserSettings {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<UserSettings> {
        debug!("loading user settings from {:?}", path.as_ref().as_os_str());
        let f = File::open(path)?;
        let buf = BufReader::new(f);
        Ok(serde_json::from_reader(buf)?)
    }

    pub fn from_file_or_default<T: AsRef<Path>>(path: T) -> UserSettings {
        match UserSettings::from_file(path) {
            Ok(u) => u,
            Err(e) => {
                warn!("failed to load user settings from file {:?}", e);
                UserSettings::default()
            },
        }
    }

    pub fn from_user_dir() -> UserSettings {
        match get_user_settings_path() {
            Ok(p) => UserSettings::from_file_or_default(p),
            Err(e) => {
                warn!("failed to get user config directory {:?}", e);
                 UserSettings::default()
                },
        }
    }

    pub fn save_to_file<T: AsRef<Path>>(&self, path: T) -> Result {
        let dir = path.as_ref().parent().ok_or(SettingsError::InvalidPath)?;

        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        if !dir.is_dir() {
            return Err(SettingsError::InvalidPath);
        }

        let w = File::create(path)?;
        serde_json::to_writer(w, self)?;

        Ok(())
    }

    pub fn save_to_user_dir(&self) -> Result {
        let path = get_user_settings_path()?;
        Ok(self.save_to_file(path)?)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{get_user_settings_path, Result, UserSettings};

    #[test]
    fn write_read_settings_file() -> Result {
        let dest = get_user_settings_path()?
            .into_os_string()
            .into_string()
            .unwrap()
            .replace(".json", "-test.json");

        let _ = fs::remove_file(&dest);

        let testee = UserSettings {
            lang: "Sindarin".to_string(),
            book_history: vec![
                "/abc/xyz/mybooks.db".to_string(),
                "/home/elrond/books/magic.db".to_string(),
            ],
        };

        testee.save_to_file(&dest)?;
        let mut loaded = UserSettings::from_file(&dest)?;

        assert_eq!(loaded, testee);

        loaded.lang = "Quenya".to_string();
        loaded
            .book_history
            .push("/home/bilbo/journey.db".to_string());

        loaded.save_to_file(&dest)?;

        let modified = UserSettings::from_file(&dest)?;

        assert_ne!(modified, testee);

        Ok(())
    }
}
