// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// This file contains all models used for the books module.
// TODO: Remove after initial implementation is done.
#![allow(dead_code)]

// Remove as soon implementation is done
use chrono::{DateTime, Utc};
use std::marker::PhantomData;

use std::{error::Error, fmt::Display};

/// All known error for the books module.

#[derive(Debug, PartialEq)]
pub enum BookError {
    /// A generic error of the books modul.
    Generic(String),
    /// Error is returned if no item with given id were found.
    NotFound(i64),
}

impl Error for BookError {}

impl Display for BookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookError::Generic(s) => s.fmt(f),
            BookError::NotFound(id) => write!(f, "did not find book with id: {id}"),
        }
    }
}

/// SortOrder defines the direction of a query.
#[derive(Debug, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl From<String> for SortOrder {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_ref() {
            "desc" => SortOrder::Desc,
            _ => SortOrder::Asc,
        }
    }
}

impl From<&str> for SortOrder {
    fn from(value: &str) -> Self {
        SortOrder::from(value.to_string())
    }
}

/// SortDescriptor describes a column and which sort order to use.
#[derive(Debug)]
pub struct SortDescriptor(String, SortOrder);

/// StoreResult a generic store result.
pub struct StoreResult<T> {
    total: u64,
    skipped: u64,
    items: Vec<T>,
}

pub struct ConfigNew;
pub struct ConfigInitialized;

/**
Configuration for searching in the BookDB.

This struct and it's logic might be a little bit complex (Builder Pattern + ZST Trait Implementation) for
its purpose, but this project is also a playground for learning rust.
*/
#[derive(Debug)]
pub struct SearchConfig<State = ConfigNew> {    
    state: PhantomData<State>,
    skip: Option<u64>,
    sort: Option<Vec<SortDescriptor>>,
    take: Option<u64>,
    text: String,
}

impl SearchConfig<ConfigNew> {
    pub fn new(txt: &str) -> SearchConfig<ConfigNew> {
        Self {
            state: PhantomData::<ConfigNew>,
            take: None,
            text: txt.to_owned(),
            skip: None,
            sort: None,
        }
    }

    pub fn build(self) -> SearchConfig<ConfigInitialized> {
        let SearchConfig { skip, take, sort, text, state: _ } = self;
        SearchConfig { skip, take, sort, text, state: PhantomData::<ConfigInitialized> }
    } 

    pub fn use_skip(mut self, skip: u64) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn use_take(mut self, take: u64) -> Self {
        self.take = Some(take);
        self
    }

    pub fn use_sort(mut self, sort: Vec<SortDescriptor>) -> Self {
        self.sort = Some(sort);
        self        
    }

}


/// BookDB provides functions to store and retrieve books from the underlying data store.
pub trait BookDB {
    fn add_book(&self, book: Book) -> Result<Book, BookError>;
    fn update_book(&self, book: &mut Book) -> Result<(), BookError>;
    fn delete_book(&self, book: &Book) -> Result<(), BookError>;
    fn delete_book_by_id(&self, id: i64) -> Result<(), BookError>;
    fn fetch_books(&self, search: &SearchConfig) -> Result<StoreResult<Book>, BookError>;

    fn get_tags(&self, pattern: &str) -> Result<StoreResult<Tag>, BookError>;
}

/// A book representation for the bookshelf application.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Book {
    pub authors: Vec<String>,
    pub cover_img: Option<Vec<u8>>,
    pub description: Option<String>,
    pub isbn: String,
    pub lang: String,
    pub tags: Option<Vec<Tag>>,
    pub title: String,
    pub sub_title: Option<String>,
    pub publisher: Option<String>,

    // Required for Database
    pub id: i64,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// A simple tag.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Tag {
    pub value: String,
    // Required for Database
    pub id: i64,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::{SearchConfig, SortOrder, SortDescriptor};

    // This test exists only to get familiar with Rust testing
    #[test]
    fn test_sort_order_from() {
        assert_eq!(SortOrder::from("any Value"), SortOrder::Asc);
        assert_eq!(SortOrder::from("asc"), SortOrder::Asc);

        let s = "deSC".to_string();
        assert_eq!(SortOrder::from(s), SortOrder::Desc);

        assert_eq!(SortOrder::from("desc"), SortOrder::Desc);
        assert_eq!(SortOrder::from("dEsC"), SortOrder::Desc);
    }

    #[test]
    fn my_test() {
        let cfg = SearchConfig::new("asdasdasd").use_skip(12).use_sort(vec![SortDescriptor("bal".to_owned(), SortOrder::Asc)]).use_take(21).build();        
        //let cfg = SearchConfig::new("asdasdasd").use_skip(12);        
        
        println!("Build search config {} {} {}", cfg.text, cfg.skip.unwrap(), cfg.take.unwrap());
        //println!("Build search config {} {}", cfg.text, cfg.skip.unwrap())   

        println!("AGAIN {} Size: {}", cfg.skip.unwrap(), std::mem::size_of_val(&cfg.state));
    }
}
