// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::books::{BookManager, self};
use crate::books::models::{Book, BookError};

#[derive(Debug, Clone, Serialize, Default)]
pub struct ApiError {
    error: String,
    code: i64,
}

// TODO: Create macro to simplify implementation
impl From<BookError> for ApiError {
    fn from(value: BookError) -> Self {
        // match value {
        //     books::models::BookError::Generic(_) => 4,
        //     books::models::BookError::NotFound => 5,
        //     books::models::BookError::DBError(_) => 6,
        //     books::models::BookError::EmptyAuthors => 7,
        // }
        Self { error: format!("{:?}", value), code: 25 }
    }
}

impl From<books::Error> for ApiError {
    fn from(value: books::Error) -> Self {            
       match value {
            books::Error::PoolAlreadyAdded => Self { error: format!("{:?}", value), code: 25 },
            books::Error::PoolNotFound => Self { error: format!("{:?}", value), code: 25 },
            books::Error::CurrentPoolNotSet => Self { error: format!("{:?}", value), code: 25 },
            books::Error::BookError(be) => be.into(),
        }
        
    }
}

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
         let d = manager.0.lock().unwrap().get_current_pool()?;
         d
     };
 
     let mut b = Book::default();
     db.add_book(&mut b)?;
 
     println!("HELLO FROM MY API {}", path);
 
     Ok("MyString".to_owned())
 }