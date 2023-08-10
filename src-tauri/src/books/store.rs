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

#[cfg(debug_assertions)]
fn create_sqlite_connection(_: &str) -> Result<Connection, BookError> {
    Ok(Connection::open_in_memory()?)
}

#[cfg(not(debug_assertions))]
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
    fn add_book(&mut self, mut book: Book) -> Result<Book, BookError> {
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

        tx.commit()?;

        Ok(book)
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
        let mut book = Book::default();
        

        Ok(Book::default())
    }
}


fn load_authors_of_book(conn: &Connection, id: &i64) -> Result<Vec<String>, BookError> {
    let query = "SELECT name FROM authors WHERE book_id = ?1 ORDER BY name ASC";

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map(&[id], |row| Ok(row.get::<usize, String>(0)?))?;
    
    let mut authors: Vec<String> = Vec::new();
    for tag in rows {
        authors.push(tag?);
    }

    Ok(authors)
}

fn load_tags_of_book(
    conn: &Connection,
    id: &i64,
) -> Result<Vec<String>, BookError> {
    let query = "SELECT tag FROM tags WHERE book_id = ?1 ORDER BY tag ASC";

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map(&[id], |row| Ok(row.get::<usize, String>(0)?))?;
    
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
            "invalid timestamp conversion".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::SqliteStore;
    use crate::books::models::{Book, BookDB, BookError};
    use crate::books::store::{load_authors_of_book, load_tags_of_book};
    use chrono::prelude::*;
    use chrono::Utc;
    use rusqlite::Connection;

    type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

    #[test]
    fn add_book_successfully() -> Result {
        let mut db = SqliteStore::new("db_file")?;
        let new_book = Book {
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

        let saved_book = db.add_book(new_book.clone())?;
        assert_eq!(new_book, saved_book);

        Ok(())
    }

    #[test]
    fn rusqlite_err() {
        let myfn = || -> Result<Connection, BookError> {
            let con = Connection::open("/var/asdh")?;
            Ok(con)
        };

        match myfn() {
            Ok(_) => println!("Connection ok"),
            Err(e) => match e {
                BookError::Generic(s) => println!("Generic error happen {}", s),
                BookError::NotFound(id) => println!("Id not found {}", id),
                BookError::DBError(e) => match e.downcast_ref::<rusqlite::Error>() {
                    Some(e) => println!("Rusqlite Error: {}", e),
                    None => println!("Some other error"),
                },
            },
        }
    }

    #[test]
    fn db_test_fn() -> Result<(), BookError> {
        let db = SqliteStore::new("db_file")?;

        println!("Got DB {:?}", db);

        // let tx = db.conn.transaction()?;
        // let vec = load_authors_of_book(&tx, &1)?;

        let vec = load_authors_of_book(&db.conn, &1)?;

        println!(">>>>>>>>>>>> {:?}", vec);

        let tags = load_tags_of_book(&db.conn, &1)?;

        println!(">>>>>>>>>>>> {:?}", tags);

        Ok(())

        // let mybook = db.add_book(Book {
        //     authors: vec![String::from("James Bond"), "Ms Money Penny".to_owned()],
        //     cover_img: None,
        //     description: Some("Supe Duper Book".to_owned()),
        //     isbn: String::from("132456789"),
        //     lang: String::from("DE"),
        //     tags: Some(vec!["Thriller".to_owned(), "Spies".to_owned()]),
        //     title: String::from("Never say never"),
        //     sub_title: None,
        //     publisher: Some("Broccoli Verlag".to_owned()),
        //     publish_date: Some(Utc::now()),
        //     id: 0,
        //     created: Utc::now()
        //         .checked_sub_signed(chrono::Duration::seconds(6400000))
        //         .unwrap(),
        //     updated: Utc::now()
        //         .checked_sub_signed(chrono::Duration::seconds(6900000))
        //         .unwrap(),
        // })?;

        // println!("Saved item {:?}", mybook);

        // let saved: (String, i64, i64) = db.conn.query_row(
        //     "SELECT title, created, updated FROM books WHERE id = ?1",
        //     params![&mybook.id],
        //     |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        // )?;

        // println!("Queryed {:?}", saved);

        // let js_str = serde_json::to_string(&mybook).unwrap();
        // println!(">>>>>>>>>> JSON: {}", js_str);

        // let new_book: Book = serde_json::from_str(&js_str).unwrap();

        // println!(
        //     "Deserialized: {:?} IS ASSERTION {:?}",
        //     new_book,
        //     cfg!(debug_assertions)
        // );

        // Ok(())

        // let conn = open_sqlite_connection("blabla.db")?;
        // println!(">>>>>>> {:?}", conn);

        // let mut stmt = conn.prepare("SELECT id, title, lang FROM books")?;

        // let nlub = stmt.query([])?.mapped(|row| {
        //     let id = row.get::<usize, u64>(0)?;
        //     println!("ID: {}", id);
        //     let mut stmt_tags = conn.prepare("SELECT value FROM tags WHERE book_id = ?")?;
        //     if let Ok(r) = stmt_tags.query_map([id], |row| Ok(row.get::<usize, String>(0).unwrap()))
        //     {
        //         let tags: Vec<String> = r.filter_map(|t| t.ok()).collect();
        //         println!("TAGS: {:?}", tags);
        //     }

        //     Ok("bla".to_owned())
        // });

        // stmt.column_names().iter().for_each(|s| {
        //     println!("{}", *s);
        // });

        // for i in nlub {
        //     println!(">>>> {}", i?);
        // }

        // Ok(())

        // let initSQL = include_str!("scripts/init.sql");
        // println!("INIT SCRIPT {}", initSQL);
        // let migrations = Migrations::new(vec![M::up(initSQL)]);

        // let mut con = Connection::open("./bla.db").unwrap();
        // match migrations.to_latest(&mut con) {
        //     Ok(_) => println!("Migrations done"),
        //     Err(e) => println!("An error occurred {}", e),
        // }

        // con.close();
    }
}
