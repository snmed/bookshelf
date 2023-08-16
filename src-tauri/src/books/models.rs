// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// This file contains all models used for the books module.
// TODO: Remove after initial implementation is done.
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt;
use std::marker::PhantomData;

use std::{error::Error, fmt::Display};

/// A simple macro to create an array of SortDescriptors.
/// Educational purpose.
#[macro_export]
macro_rules! sort_desc {
    (@Ord $ord:literal) => {$ord.into()};
    (@Ord $ord:expr) => {$ord};
    ($($col:expr, $ord:expr),+) => {
        vec![
        $(
            crate::books::models::SortDescriptor($col.into(), sort_desc!(@Ord $ord))
        ),+
        ]
    };

}


/// All known error for the books module.
/// Probably use crate `thiserror` as soon as familiar
/// enough with error handling and implementation.
#[non_exhaustive]
#[derive(Debug)]
pub enum BookError {
    /// A generic error of the books modul.
    Generic(String),
    /// Error is returned if no item with given id were found.
    NotFound,
    /// An error returned from the underlying database runtime.
    DBError(Box<dyn std::error::Error>),
    /// An error if authors is empty.
    EmptyAuthors,
}

impl Error for BookError {}
unsafe impl Send for BookError {}
unsafe impl Sync for BookError {}

pub type Result<T, E = BookError> = core::result::Result<T, E>;

impl Display for BookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookError::Generic(s) => s.fmt(f),
            BookError::NotFound => write!(f, "Did not find item with given id"),
            BookError::DBError(e) => write!(f, "Database error: {}", *e),
            BookError::EmptyAuthors => write!(f, "Book requires at least one author"),
        }
    }
}

/// SortOrder defines the direction of a query.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct SortDescriptor(pub String, pub SortOrder);

/// StoreResult a generic store result.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct StoreResult<T> {
    pub total: u64,
    pub skipped: u64,
    pub items: Vec<T>,
}

pub struct ConfigNew;
pub struct ConfigInitialized;

/**
Configuration for searching in the BookDB.

This struct and it's logic might be a little bit complex (Builder Pattern + ZST Trait Implementation) for
its purpose, but this project is also a playground for learning rust.
*/
#[derive(Deserialize)]
pub struct SearchConfig<State = ConfigNew> {
    #[serde(skip)]
    state: PhantomData<State>,
    skip: Option<u64>,
    sort: Option<Vec<SortDescriptor>>,
    take: Option<u64>,
    text: String,
}

impl<State> fmt::Debug for SearchConfig<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchConfig")
            .field("state", &self.state)
            .field("skip", &self.skip)
            .field("sort", &self.sort)
            .field("take", &self.take)
            .field("text", &self.text)
            .finish()
    }
}

impl<T: AsRef<str>> From<T> for SearchConfig<ConfigInitialized> {
    fn from(value: T) -> Self {
        SearchConfig::new(value.as_ref()).build()
    }
}

impl<State> AsRef<SearchConfig<State>> for SearchConfig<State> {
    fn as_ref(&self) -> &SearchConfig<State> {
        self
    }
}

impl From<SearchConfig<ConfigNew>> for SearchConfig<ConfigInitialized> {
    fn from(value: SearchConfig<ConfigNew>) -> Self {
        value.build()
    }
}

// Implementation for the ConfigNew state.
// As stated before, this is only for educational purpose and could
// be made easier.
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
        let SearchConfig {
            skip,
            take,
            sort,
            text,
            state: _,
        } = self;
        SearchConfig {
            skip,
            take,
            sort,
            text,
            state: PhantomData::<ConfigInitialized>,
        }
    }

    /// How many pages should be skipped. Only used if
    /// also `use_take` were specified. Example:
    /// If `use_take(20).use_skip_page(3)` were called,
    /// first 60 items will be skipped.
    pub fn use_skip_page(mut self, skip: u64) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Specifies how many items should be returned.
    pub fn use_take(mut self, take: u64) -> Self {
        self.take = Some(take);
        self
    }

    /// Define how items should be sorted.
    pub fn use_sort(mut self, sort: Vec<SortDescriptor>) -> Self {
        self.sort = Some(sort);
        self
    }
}

impl SearchConfig<ConfigInitialized> {
    pub fn get_take(&self) -> Option<&u64> {
        self.take.as_ref()
    }

    pub fn get_skip_page(&self) -> Option<&u64> {
        self.skip.as_ref()
    }

    pub fn get_sort_desc(&self) -> Option<&Vec<SortDescriptor>> {
        self.sort.as_ref()
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}

/// BookDB provides functions to store and retrieve books from the underlying data store.
/// For me as beginner, I use [core::Result] to get familiar with rust std. But in future,
/// I might use a type alias like `type Result<T, E = BookError> = core::Result<T, E>;`.
pub trait BookDB {
    fn add_book(&mut self, book: &mut Book) -> Result<()>;
    fn get_book<T: Borrow<i64>>(&mut self, id: T) -> Result<Book>;
    fn update_book(&mut self, book: &mut Book) -> Result<()>;
    fn delete_book(&mut self, book: &Book) -> Result<()>;
    fn delete_book_by_id<T: Borrow<i64>>(&mut self, id: T) -> Result<()>;
    fn fetch_books(
        &mut self,
        search: SearchConfig<ConfigInitialized>,
    ) -> Result<StoreResult<Book>>;

    fn get_tags(&mut self, search: SearchConfig<ConfigInitialized>) -> Result<StoreResult<String>>;
    fn get_authors(
        &mut self,
        search: SearchConfig<ConfigInitialized>,
    ) -> Result<StoreResult<String>>;
}

/// A book representation for the bookshelf application.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Book {
    pub authors: Vec<String>,
    pub cover_img: Option<String>,
    pub description: Option<String>,
    pub isbn: String,
    pub lang: String,
    pub tags: Option<Vec<String>>,
    pub title: String,
    pub sub_title: Option<String>,
    pub publisher: Option<String>,
    pub publish_date: Option<DateTime<Utc>>,

    // Required for Database
    pub id: i64,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}



#[cfg(test)]
mod tests {
    use super::SortOrder;

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
}
