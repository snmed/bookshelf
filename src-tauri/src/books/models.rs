// This file contains all models used for the books module.

#[allow(dead_code)]

use std::{error::Error, fmt::Display};

use chrono::{DateTime, Utc};

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

/// BookDB provides functions to store and retrieve books from the underlying data store.
pub trait BookDB {
    fn add_book(book: &mut Book) -> Result<&mut Book, BookError>;
    fn update_book(book: &mut Book) -> Result<(), BookError>;
    fn delete_book(book: &Book) -> Result<(), BookError>;
    fn delete_book_by_id(id: i64) -> Result<(), BookError>;
    // TODO: Add more functions to the BookDB trait
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
