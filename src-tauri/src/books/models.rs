// This file contains all models used for the books module.

/// A book representation for the bookshelf application.
#[derive(Debug, Default)]
pub struct Book {
    title: String,
    sub_title: Option<String>,
    description: Option<String>,


}