// This file contains all models used for the books module.

// Remove as soon implementation is done
use chrono::{DateTime, Utc};
#[allow(dead_code)]
use std::{error::Error, fmt::Display};

// All known error for the books module.

#[derive(Debug, PartialEq)]
pub enum BookError {
    Generic(String),
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
pub struct SortDescriptor(String, SortOrder);

/// StoreResult a generic store result.
pub struct StoreResult<T> {
    total: u64,
    skipped: u64,
    items: Vec<T>,
}

pub struct SearchConfig {
    skip: Option<u64>,
    sort: Option<Vec<SortDescriptor>>
}

/// BookDB provides functions to store and retrieve books from the underlying data store.
pub trait BookDB {
    fn add_book(book: Book) -> Result<Book, BookError>;
    fn update_book(book: &mut Book) -> Result<(), BookError>;
    fn delete_book(book: &Book) -> Result<(), BookError>;
    fn delete_book_by_id(id: i64) -> Result<(), BookError>;
    fn fetch_books(search: &str) -> Result<StoreResult<Book>, BookError>;

    fn get_tags() -> Result<StoreResult<Tag>, BookError>;
}

/// A book representation for the bookshelf application.
#[derive(Debug, Default, PartialEq)]
pub struct Book {
    authors: Vec<String>,
    cover_img: Option<Vec<u8>>,
    description: Option<String>,
    isbn: String,
    lang: String,
    tags: Option<Vec<Tag>>,
    title: String,
    sub_title: Option<String>,

    // Required for Database
    id: i64,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

/// A simple tag.
#[derive(Debug, Default, PartialEq)]
pub struct Tag {
    value: String,
    // Required for Database
    id: i64,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use crate::books::models::SortOrder;

    // This test is only to get familiar with Rust testing
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

        let myfn = |s: String| println!("intern function {}!", s);

        let txt = String::from("John");

        myfn(txt.clone());
        println!("OUTSIDE {}", txt);

        
    }
}
