// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

/// Creates a a From trait for given types and error enum.
/// 
/// ```
/// pub enum SettingsError {
///     UserDirNotFound,
///     InvalidPath,
///     SerdeError(serde_json::Error),
///     IoError(io::Error),
/// }
///
/// from_err!(SettingsError, serde_json::Error, SerdeError);
/// from_err!(SettingsError, io::Error, IoError);
///```
/// Will create:
/// ```
/// impl From<serde_json::Error> for SettingsError {
///     fn from(value: serde_json::Error) -> Self {
///         Self::SerdeError(value)
///     }
/// }
///
/// impl From<io::Error> for SettingsError {
///     fn from(value: io::Error) -> Self {
///         Self::IoError(value)
///     }
/// }
/// ```
#[macro_export]
macro_rules! from_err {
    ($for:ty, $from:ty, $to:ident) => {
        impl From<$from> for $for {
            fn from(value: $from) -> Self {
                Self::$to(value)
            }
        }
    };
    ($for:ty, $from:ty, to $to:ident) => {
        impl From<$from> for $for {
            fn from(value: $from) -> Self {
               Self::$to 
            }
        }
    };
}