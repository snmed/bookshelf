use std::collections::HashMap;


use self::models::{BookDB, BookError};
use crate::pool::{PoolItem, PoolManager};
use crate::{from_err};

// Module declarations
pub mod models;
mod store;

#[derive(Debug)]
pub enum Error {
    PoolAlreadyAdded,
    PoolNotFound,
    CurrentPoolNotSet,
    BookError(BookError),
}
from_err!(Error, BookError, BookError);

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

pub type BookPool = PoolManager<Box<dyn BookDB>>;

#[derive(Default)]
pub struct BookManager {
    book_db_pools: HashMap<String, BookPool>,
    current: Option<String>,
}

impl BookManager {
    pub fn add_pool<K: AsRef<str>>(&mut self, pool_name: K, pool: BookPool) -> Result {
        todo!()
    }

    pub fn remove_pool<T: AsRef<str>>(&mut self, pool_name: T) -> Option<BookPool> {
        match self.book_db_pools.remove_entry(pool_name.as_ref()) {
            Some(entry) => {
                if self.current.is_some() && entry.0.as_str() == pool_name.as_ref() {
                    self.current = None;
                }
                Some(entry.1)
            }
            None => None,
        }
    }

    pub fn is_current_pool_set(&self) -> bool {
        self.current.is_some()
    }

    pub fn set_current_pool<T: AsRef<str>>(&mut self, pool_name: T) -> Result {
        if self.book_db_pools.contains_key(pool_name.as_ref()) {
            self.current.replace(pool_name.as_ref().to_string());
            return Ok(());
        }

        Err(Error::PoolNotFound)
    }

    pub fn get_pools(&self) -> Vec<&str> {
        self.book_db_pools.keys().map(|k| k.as_str()).collect()
    }

    pub fn get_current_pool(&self) -> Result<PoolItem<Box<dyn BookDB>>> {
        match self.current.as_ref() {
            Some(s) => Ok(self
                .book_db_pools
                .get(s)
                .ok_or(Error::PoolNotFound)?
                .get_pool_item()),
            None => Err(Error::CurrentPoolNotSet),
        }
    }
}
