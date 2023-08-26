// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::books::models::{Book, BookError};
use crate::books::{self, BookManager};

// macro_rules! from_err_api {
//     (@Internal $value:ident  $error:literal $code:literal ) => {
//         Self {error: format!("{:?}", $error), code: $code }
//     };
//     (@Internal $value:ident  $code:literal ) => {
//         Self {error: format!("{:?}", $value), code: $code }
//     };
//     // (@Internal $value:ident $other:expr ) => {
//     //     $other
//     // };
//     (@Enum $value:ident $enum:pat $(if $pred:expr)* => $($to2:tt)*) => {
//         $enum $(if $pred)* => from_err_api!(@Internal $value $($to)*)
//     };
//     ($from:ty,
//         $($enum:pat $(if $pred:expr)* => $($to:tt)* ),+
//     ) => {
//         impl From<$from> for ApiError {
//             fn from(value: $from) -> Self {
//                match value {
//                     $(
//                         //$enum $(if $pred)* => from_err_api!(@Internal value $to)
//                         from_err_api!(@Enum value $enum $(if $pred)* => $($to)* )
//                     ),+
//                 }
//             }
//         }
//     };

// }

macro_rules! from_err_api {
    ($code:literal) => {
        ApiError {error: "Unspecified API Error".to_string(), code: $code }
    };
    
    ($error:expr, $code:expr) => {
        ApiError {error: $error, code: $code }
    };  
    ($from:ty, $($enum:pat $(if $pred:expr)* => $result:expr),* ) => {
        impl From<$from> for ApiError {
                    fn from(value: $from) -> Self {
                    match value {
                        $(
                            $enum $(if $pred)* => $result
                        ),*
                    }
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
    BookError::Generic(e) => from_err_api!(e, 5),
    BookError::NotFound => from_err_api!(5), 
    _ => Self {..Default::default()}
);

// from_err_api!(BookError,
//     BookError::Generic(_) => 4,
//     BookError::NotFound => 5,
//     BookError::DBError(_) => 6,
//     BookError::EmptyAuthors => 7
// );

// from_err_api!(books::Error,
//     books::Error::PoolAlreadyAdded => 1,
//     books::Error::PoolNotFound => 2,
//     books::Error::CurrentPoolNotSet => 3,
//     books::Error::BookError(_) =>  {"sdfdsfdsf" 44},
// );

// TODO: Create macro to simplify implementation
// impl From<BookError> for ApiError {
//     fn from(value: BookError) -> Self {
//         // match value {
//         //     books::models::BookError::Generic(_) => 4,
//         //     books::models::BookError::NotFound => 5,
//         //     books::models::BookError::DBError(_) => 6,
//         //     books::models::BookError::EmptyAuthors => 7,
//         // }
//         Self { error: format!("{:?}", value), code: 25 }
//     }
// }

// impl From<books::Error> for ApiError {
//     fn from(value: books::Error) -> Self {
//        match value {
//             books::Error::PoolAlreadyAdded => Self { error: format!("{:?}", value), code: 25 },
//             books::Error::PoolNotFound => Self { error: format!("{:?}", value), code: 25 },
//             books::Error::CurrentPoolNotSet => Self { error: format!("{:?}", value), code: 25 },
//             //books::Error::BookError(be) => be.into(),
//             _ => Self { error: format!("{:?}", value), code: 25 }
//         }

//     }
// }

type Result<T = (), E = ApiError> = std::result::Result<T, E>;

/*******************************************************
 *
 * Book API
 *
 ******************************************************/

#[derive(Default)]
pub struct BookManagerState(Arc<Mutex<BookManager>>);

#[tauri::command]
pub async fn create_book_db(path: String, manager: State<'_, BookManagerState>) -> Result<String> {
    let mut db = {
        let d = manager.0.lock().unwrap().get_current_pool().unwrap();
        d
    };

    let mut b = Book::default();
    db.add_book(&mut b).unwrap();

    println!("HELLO FROM MY API {}", path);

    Ok("MyString".to_owned())
}

#[cfg(test)]
mod tests {
    use crate::books::models::BookError;

    use super::ApiError;

    #[test]
    fn bla() {
        //let my: ApiError = BookError::NotFound.into();
        let my: ApiError = BookError::Generic("Super Duper".to_string()).into();
        println!(">>>>>>>>>>>>>>< {:?}", my);
        println!("--> {:?}", from_err_api!("asdasdasdsad".to_owned(), 42));
        println!("--> {:?}", from_err_api!("asdasdasdsad".to_owned(), 42));
    }
}
