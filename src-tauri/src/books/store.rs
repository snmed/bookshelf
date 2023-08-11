// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// TODO: Remove after initial implementation is done.
#![allow(dead_code)]

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{named_params, params, Connection, MappedRows, Row, Transaction};
use rusqlite_migration::{Migrations, M};

use super::models::{Book, BookDB, BookError, SearchConfig, StoreResult};

/// Opens or creates a new books database and returns it.
fn open_sqlite_connection(db_file: &str) -> Result<Connection, BookError> {
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

// #[cfg(debug_assertions)]
// fn create_sqlite_connection(_: &str) -> Result<Connection, BookError> {
//     Ok(Connection::open_in_memory()?)
// }

// #[cfg(not(debug_assertions))]
fn create_sqlite_connection(db_file: &str) -> Result<Connection, BookError> {
    Ok(Connection::open(db_file)?)
}

#[derive(Debug)]
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    fn new(db_file: &str) -> Result<Self, BookError> {
        Ok(Self {
            conn: open_sqlite_connection(db_file)?,
        })
    }
}

impl BookDB for SqliteStore {
    /// Add a new book to the store.
    /// TODO: Write a unit test to ensure functionality.
    fn add_book(&mut self, book: &mut Book) -> Result<(), BookError> {
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
            ":pubd": book.publish_date
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

    fn update_book(&mut self, book: &mut Book) -> Result<(), BookError> {
        todo!()
    }

    fn delete_book(&mut self, book: &Book) -> Result<(), BookError> {
        todo!()
    }

    fn delete_book_by_id(&mut self, id: &i64) -> Result<(), BookError> {
        todo!()
    }

    fn fetch_books(&mut self, search: &SearchConfig) -> Result<StoreResult<Book>, BookError> {
        todo!()
    }

    fn get_tags(&mut self, pattern: &str) -> Result<StoreResult<String>, BookError> {
        todo!()
    }

    fn get_authors(&mut self, search: &SearchConfig) -> Result<StoreResult<String>, BookError> {
        todo!()
    }

    fn get_book(&mut self, id: &i64) -> Result<Book, BookError> {
        let query = r#"SELECT id, cover_img, description, isbn, lang, title, sub_title,
         publisher, publish_date, created, updated FROM books WHERE id = ?1"#;

        let book = self.conn.query_row(query, [id], |row| {
            Ok(Book {
                authors: load_authors_of_book(&self.conn, id)?,
                cover_img: row.get("cover_img")?,
                description: row.get("description")?,
                isbn: row.get("isbn")?,
                lang: row.get("lang")?,
                tags: load_tags_of_book(&self.conn, id).map(|v| match v.len() {
                    0 => None,
                    _ => Some(v)
                })?,
                title: row.get("title")?,
                sub_title: row.get("sub_title")?,
                publisher: row.get("publisher")?,
                publish_date: row.get("publish_date")?,
                id: row.get("id")?,
                created: convert_timestamp(row.get::<&str, i64>("created")?).expect("Conversion database integer to DateTime failed"),
                updated: convert_timestamp(row.get::<&str, i64>("updated")?).expect("Conversion database integer to DateTime failed"),
            })
        })?;

        Ok(book)
    }
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
        BookError::DBError(value.into())
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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::thread;
    use std::time::Duration;

    use super::SqliteStore;
    use crate::books::models::{Book, BookDB, BookError};
    use crate::books::store::{load_authors_of_book, load_tags_of_book};
    use chrono::prelude::*;
    use chrono::Utc;
    

    type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

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
        let saved_book = db.get_book(&new_book.id)?;

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

    #[test]
    fn db_test_fn() -> Result<(), BookError> {

        let handle = thread::spawn(|| -> Result<(), BookError> {
            let mut db = SqliteStore::new("blabla.db")?;
            println!("Got DB {:?}", db);
            thread::sleep(Duration::from_millis(500));
            
            let mut b = Book {
                title: "Mein Krampf mit Rust".to_owned(),
                .. Book::default()
            };

            db.add_book(&mut b)?;

            println!("Book saved in thread {:?}", b);

            Ok(())

        });
        
        

        let mut db2 = SqliteStore::new("blabla.db")?;

        println!("Got DB 2 {:?}", db2);

        let mut b = Book {
            title: "Mein Krampf mit Rust 2".to_owned(),
            .. Book::default()
        };

        db2.add_book(&mut b)?;


        handle.join();

        Ok(())

    }
}
