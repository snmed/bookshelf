use std::collections::HashMap;
use std::io;

use self::models::{BookDB, BookError};
use crate::pool::PoolManager;
use crate::{macros, from_err};

// Module declarations
pub mod models;
mod store;



#[derive(Debug)]
pub enum Error {
    PoolAlreadyAdded,
    BookError(BookError)
}

from_err!(Error, BookError, BookError);

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

pub type BookPool = PoolManager<Box<dyn BookDB>>;


pub struct BookManager {
    book_db_pools: HashMap<String, BookPool>,
    current: String
}

impl BookManager {
    pub fn add_pool<K: AsRef<str>>(&mut self, pool_name: K, pool: BookPool) -> Result {
        todo!()
    }
}