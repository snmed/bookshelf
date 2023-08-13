// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// TODO: Remove after initial implementation is done.
#![allow(dead_code)]

use std::borrow::Borrow;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{named_params, params, params_from_iter, Connection};
use rusqlite_migration::{Migrations, M};

use super::models::{
    Book, BookDB, BookError, ConfigInitialized, Result, SearchConfig, SearchParam, StoreResult, SortOrder, SortDescriptor,
};

const SELECT_BOOKS_QUERY: &str = r#"SELECT id, cover_img, description, isbn, lang, title, sub_title,
publisher, publish_date, created, updated FROM books"#;

/// Opens or creates a new books database and returns it.
fn open_sqlite_connection(db_file: &str) -> Result<Connection> {
    // Add all required sql scripts to the migrator
    let mut scripts = vec![M::up(include_str!("scripts/init.sql"))];

    // Add only for debug mode dummy data
    if cfg!(debug_assertions) {
        scripts.push(M::up(include_str!("scripts/dummy_data.sql")));
    }

    let mut conn = create_sqlite_connection(db_file)?;
    let migrations = Migrations::new(scripts);

    migrations.to_latest(&mut conn)?;

    conn.pragma_update(None, "journal_mode", "wal")?;
    conn.pragma_update(None, "synchronous", "normal")?;
    conn.pragma_update(None, "foreign_keys", "on")?;

    Ok(conn)
}

#[cfg(debug_assertions)]
fn create_sqlite_connection(_: &str) -> Result<Connection> {
    Ok(Connection::open_in_memory()?)
}

#[cfg(not(debug_assertions))]
fn create_sqlite_connection(db_file: &str) -> Result<Connection> {
    Ok(Connection::open(db_file)?)
}

#[derive(Debug)]
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    fn new(db_file: &str) -> Result<Self> {
        Ok(Self {
            conn: open_sqlite_connection(db_file)?,
        })
    }
}

impl BookDB for SqliteStore {
    /// Add a new book to the store.
    /// TODO: Write a unit test to ensure functionality.
    fn add_book(&mut self, book: &mut Book) -> Result<()> {
        let tx = self.conn.transaction()?;

        let mut books_stmt = tx.prepare(r#"INSERT INTO books (cover_img, description, isbn, lang, title, sub_title, publisher, publish_date, created, updated)
        VALUES (:img, :desc, :isbn, :lang , :title, :subt, :pub, :pubd, unixepoch(), unixepoch())"#)?;

        let book_id = books_stmt.insert(named_params! {
            ":img": book.cover_img,
            ":desc": book.description,
            ":isbn": book.isbn,
            ":lang": book.lang,
            ":title": book.title,
            ":subt": book.sub_title,
            ":pub": book.publisher,
            ":pubd": book.publish_date.as_ref().map(|d| d.timestamp())
        })?;
        drop(books_stmt);

        if book_id <= 0 {
            return Err(BookError::Generic(format!(
                "return row id is invalid: {}",
                book_id
            )));
        }
        book.id = book_id;

        {
            let mut authors_stmt =
                tx.prepare("INSERT INTO authors (book_id, name) VALUES (?1, ?2)")?;
            for author in &book.authors {
                authors_stmt.execute(params![book_id, author])?;
            }

            {
                if let Some(tags) = &book.tags {
                    let mut tags_stmt =
                        tx.prepare("INSERT INTO tags (book_id, tag) VALUES (?1, ?2)")?;
                    for tag in tags {
                        tags_stmt.execute(params![book_id, tag])?;
                    }
                }
            }
        }

        let dates: (i64, i64) = tx.query_row(
            "SELECT created, updated FROM books WHERE id = ?1",
            [&book_id],
            |row| Ok((row.get::<usize, i64>(0)?, row.get::<usize, i64>(1)?)),
        )?;

        book.created = convert_timestamp(dates.0)?;
        book.updated = convert_timestamp(dates.1)?;

        book.authors.sort();
        if let Some(tags) = book.tags.as_mut() {
            tags.sort();
        }

        tx.commit()?;

        Ok(())
    }

    fn update_book(&mut self, book: &mut Book) -> Result<()> {
        let query = r#"UPDATE books SET cover_img = :img, description = :desc, isbn = :isbn, lang = :lang, 
            title = :title, sub_title = :sub, publisher = :pub, 'publish_date' = :pdate, updated = unixepoch() WHERE id = :id"#;

        let tx = self.conn.transaction()?;

        tx.execute(
            query,
            named_params! {
                ":img": book.cover_img,
                ":desc": book.description,
                ":isbn": book.isbn,
                ":lang": book.lang,
                ":title": book.title,
                ":sub": book.sub_title,
                ":pub": book.publisher,
                ":pdate": book.publish_date.as_ref().map(|d| d.timestamp()),
                ":id": book.id
            },
        )?;

        update_book_tags(&tx, book)?;
        update_book_authors(&tx, book)?;

        tx.commit()?;

        Ok(())
    }

    fn delete_book(&mut self, book: &Book) -> Result<()> {
        self.delete_book_by_id(book.id)
    }

    fn delete_book_by_id<T>(&mut self, id: T) -> Result<()>
    where
        T: Borrow<i64>,
    {
        self.conn
            .execute("DELETE FROM books WHERE id = ?", [id.borrow()])?;
        Ok(())
    }

    fn fetch_books<T: AsRef<SearchConfig<ConfigInitialized>>>(
        &mut self,
        search: T,
    ) -> Result<StoreResult<Book>> {
        todo!()
    }

    fn get_tags(&mut self, pattern: &str) -> Result<StoreResult<String>> {
        todo!()
    }

    fn get_authors<T: AsRef<SearchConfig<ConfigInitialized>>>(
        &mut self,
        search: T,
    ) -> Result<StoreResult<String>> {
        let query = r"SELECT DISTINCT name FROM authors".to_owned();

        todo!()
    }

    fn get_book<T>(&mut self, id: T) -> Result<Book>
    where
        T: Borrow<i64>,
    {
        // let query = r#"SELECT id, cover_img, description, isbn, lang, title, sub_title,
        //  publisher, publish_date, created, updated FROM books WHERE id = ?1"#;

        let query = format!("{} WHERE id = ?1", SELECT_BOOKS_QUERY);

        let book = self.conn.query_row(&query, [id.borrow()], |row| {
            Ok(Book {
                authors: load_authors_of_book(&self.conn, id.borrow())?,
                cover_img: row.get("cover_img")?,
                description: row.get("description")?,
                isbn: row.get("isbn")?,
                lang: row.get("lang")?,
                tags: load_tags_of_book(&self.conn, id.borrow()).map(|v| match v.len() {
                    0 => None,
                    _ => Some(v),
                })?,
                title: row.get("title")?,
                sub_title: row.get("sub_title")?,
                publisher: row.get("publisher")?,
                publish_date: row
                    .get::<&str, i64>("publish_date")
                    .map(|r| {
                        convert_timestamp(r)
                            .expect("Conversion database integer to DateTime failed")
                    })
                    .ok(),
                id: row.get("id")?,
                created: convert_timestamp(row.get::<&str, i64>("created")?)
                    .expect("Conversion database integer to DateTime failed"),
                updated: convert_timestamp(row.get::<&str, i64>("updated")?)
                    .expect("Conversion database integer to DateTime failed"),
            })
        })?;

        Ok(book)
    }
}

fn update_book_authors(conn: &Connection, book: &mut Book) -> Result<()> {
    if book.authors.is_empty() {
        return Err(BookError::EmptyAuthors);
    }

    conn.execute("DELETE FROM authors WHERE book_id = ?1", [&book.id])?;
    book.authors.sort();

    let mut stmt = conn.prepare("INSERT INTO authors (book_id, name) VALUES (:id, :name)")?;
    for a in &book.authors {
        stmt.execute(named_params! {":id": &book.id, ":name": a})?;
    }

    Ok(())
}

fn update_book_tags(conn: &Connection, book: &mut Book) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE book_id = ?1", [&book.id])?;

    match book.tags.as_mut() {
        None => return Ok(()),
        Some(t) if t.is_empty() => {
            book.tags = None;
            return Ok(());
        }
        Some(_) => (),
    }

    book.tags.as_mut().unwrap().sort();
    book.tags.as_mut().unwrap().dedup();

    let mut stmt = conn.prepare("INSERT INTO tags (book_id, tag) VALUES (:id, :tag)")?;
    for t in book.tags.as_ref().unwrap() {
        stmt.execute(named_params! { ":id": &book.id, ":tag": t })?;
    }

    Ok(())
}

fn load_authors_of_book(conn: &Connection, id: &i64) -> Result<Vec<String>, rusqlite::Error> {
    let query = "SELECT name FROM authors WHERE book_id = ?1 ORDER BY name ASC";

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([id], |row| row.get::<usize, String>(0))?;

    let mut authors: Vec<String> = Vec::new();
    for tag in rows {
        authors.push(tag?);
    }

    Ok(authors)
}

fn load_tags_of_book(conn: &Connection, id: &i64) -> Result<Vec<String>, rusqlite::Error> {
    let query = "SELECT tag FROM tags WHERE book_id = ?1 ORDER BY tag ASC";

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([id], |row| row.get::<usize, String>(0))?;

    let mut tags: Vec<String> = Vec::new();
    for tag in rows {
        tags.push(tag?);
    }

    Ok(tags)
}

impl From<rusqlite::Error> for BookError {
    fn from(value: rusqlite::Error) -> Self {
        // Todo: If necessary transform [rusqlite::Error] errors into database agnostic errors.
        match value {
            rusqlite::Error::QueryReturnedNoRows => BookError::NotFound,
            _ => BookError::DBError(value.into()),
        }
    }
}

impl From<rusqlite_migration::Error> for BookError {
    fn from(value: rusqlite_migration::Error) -> Self {
        // Todo: If necessary transform [rusqlite_migration::Error] errors into database agnostic errors.
        BookError::DBError(value.into())
    }
}

fn convert_timestamp(timestamp: i64) -> Result<DateTime<Utc>, BookError> {
    match Utc.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => Ok(dt),
        _ => Err(BookError::Generic(
            "Invalid timestamp conversion".to_owned(),
        )),
    }
}

pub trait ToQuery {
    fn to_query(&self) -> String;
}

impl ToQuery for SortDescriptor {
    fn to_query(&self) -> String {
        match self.1 {
            SortOrder::Asc => format!("{} ASC", self.0),
            SortOrder::Desc => format!("{} DESC", self.0),
        }
    }
}

// // Implementation for the ConfigInitialized state.
// impl SearchConfig<ConfigInitialized> {
//     /// Builds a sqlite query with given base query and a function which
//     /// takes a search text and returns part of a where clause.
//     /// For example: `id = ?1 AND name LIKE ?2`
//     pub fn as_sqlite_query<Q, F>(&self, base_query: Q, f: F) -> String
//     where
//         Q: AsRef<str>,
//         F: FnOnce(&str) -> String,
//     {
//         let mut query = format!("{} ", base_query.as_ref());
//         if !self.text.is_empty() {
//             query.push_str(format!("WHERE {} ", f(&self.text)).as_ref());
//         }

//         if let Some(sort) = self.sort.as_ref() {
//             query.push_str("ORDER BY ");
//             query.push_str(
//                 sort.iter()
//                     .map(|d| d.to_query())
//                     .collect::<Vec<String>>()
//                     .join(", ")
//                     .as_ref(),
//             );
//             query.push(' ');
//         }

//         if let Some(l) = &self.take {
//             query.push_str(format!("LIMIT {} ", l).as_ref());
//         }

//         if let Some(o) = &self.skip {
//             query.push_str(format!("OFFSET {}", o).as_ref());
//         }

//         query
//     }
// }

#[cfg(test)]
mod tests {
    use super::SqliteStore;
    use crate::books::models::{Book, BookDB};
    use crate::books::models::{ConfigInitialized, SearchConfig};
    use chrono::prelude::*;
    use chrono::Utc;
    use std::error::Error;

    type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

    #[test]
    fn fetch_books() -> Result {
        let mut db = SqliteStore::new("db_file")?;

        let books = db.fetch_books(SearchConfig::new("").build())?;
        assert_eq!(books.total, 3);
        assert_eq!(books.skipped, 0);
        assert_eq!(books.items.len(), 3);

        let partial_books = db.fetch_books(SearchConfig::new("").use_skip(1).build())?;
        assert_eq!(partial_books.total, 2);
        assert_eq!(partial_books.skipped, 1);
        assert_eq!(partial_books.items.len(), 2);

        Ok(())
    }

    #[test]
    fn delete_book_successfully() -> Result {
        let mut db = SqliteStore::new("db_file")?;

        db.delete_book_by_id(1)?;
        assert!(db.get_book(1).is_err());

        Ok(())
    }

    #[test]
    fn add_book_successfully() -> Result {
        let mut db = SqliteStore::new("db_file")?;
        let mut new_book = Book {
            authors: vec![String::from("Schiller"), "Goethe".to_owned()],
            cover_img: None,
            description: Some("Most loved and famous book ever!".to_owned()),
            isbn: String::from("123456789"),
            lang: String::from("DE"),
            tags: Some(vec!["Classic".to_owned(), "Poem".to_owned()]),
            title: String::from("The Famous One"),
            sub_title: None,
            publisher: Some("Plato Verlag".to_owned()),
            publish_date: Some(Utc.with_ymd_and_hms(1743, 1, 12, 13, 14, 44).unwrap()),
            id: 123465798, // Should never be set or inserted
            created: Utc::now()
                .checked_sub_signed(chrono::Duration::seconds(1000000))
                .unwrap(),
            updated: Utc::now()
                .checked_sub_signed(chrono::Duration::seconds(1000000))
                .unwrap(),
        };

        db.add_book(&mut new_book)?;
        let saved_book = db.get_book(new_book.id)?;

        assert_eq!(new_book.title, saved_book.title);
        assert_eq!(new_book.sub_title, saved_book.sub_title);
        assert_eq!(new_book.isbn, saved_book.isbn);
        assert_eq!(new_book.cover_img, saved_book.cover_img);
        assert_eq!(new_book.description, saved_book.description);
        assert_eq!(new_book.lang, saved_book.lang);
        assert_eq!(new_book.publisher, saved_book.publisher);
        assert_eq!(new_book.publish_date, saved_book.publish_date);

        assert_eq!(new_book.created, saved_book.created);
        assert_eq!(new_book.updated, saved_book.updated);

        assert_eq!(new_book.authors, saved_book.authors);
        assert_eq!(new_book.tags, saved_book.tags);

        Ok(())
    }
}
