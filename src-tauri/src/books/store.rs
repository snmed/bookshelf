// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

// TODO: Remove after initial implementation is done.
#![allow(dead_code)]

use std::borrow::Borrow;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{named_params, params, params_from_iter, Connection, ToSql};
use rusqlite_migration::{Migrations, M};

use super::models::{
    Book, BookDB, BookError, ConfigInitialized, Result, SearchConfig, SortOrder, StoreResult,
};

const SELECT_BOOKS_QUERY: &str = r#"SELECT id, cover_img, description, isbn, lang, title, sub_title,
publisher, publish_date, created, updated FROM books"#;
const SELECT_AUTHORS_QUERY: &str = "SELECT DISTINCT name FROM authors";
const SELECT_TAGS_QUERY: &str = "SELECT DISTINCT tag FROM tags";


/// Maps a sqlite row to a Book.
/// Requires a connection reference,
macro_rules! map_sqlite_row_to_book {
    ($conn:expr, $row:ident) => {{
        let id: i64 = $row.get("id")?;
        Book {
            authors: load_authors_of_book($conn, &id)?,
            cover_img: $row.get("cover_img")?,
            description: $row.get("description")?,
            isbn: $row.get("isbn")?,
            lang: $row.get("lang")?,
            tags: load_tags_of_book($conn, &id).map(|v| match v.len() {
                0 => None,
                _ => Some(v),
            })?,
            title: $row.get("title")?,
            sub_title: $row.get("sub_title")?,
            publisher: $row.get("publisher")?,
            publish_date: $row
                .get::<&str, i64>("publish_date")
                .map(|r| {
                    convert_timestamp(r).expect("Conversion database integer to DateTime failed")
                })
                .ok(),
            id,
            created: convert_timestamp($row.get::<&str, i64>("created")?)
                .expect("Conversion database integer to DateTime failed"),
            updated: convert_timestamp($row.get::<&str, i64>("updated")?)
                .expect("Conversion database integer to DateTime failed"),
        }
    }};
}

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

    fn fetch_books(
        &mut self,
        search: SearchConfig<ConfigInitialized>,
    ) -> Result<StoreResult<Book>> {
        todo!()
    }

    /// Gets a result of stored tags.
    /// TODO: USe FTS5 for improve the performance of this naive implementation.
    fn get_tags(&mut self, search: SearchConfig<ConfigInitialized>) -> Result<StoreResult<String>> {
        let mut builder = QueryBuilder::new(&SELECT_TAGS_QUERY, search.as_ref());
        builder.use_where_clause(|txt| {
            ("tag LIKE ?".to_owned(), vec![format!("%{}%", txt)])
        });

        let mut authors: StoreResult<String> = StoreResult::default();
        builder.fetch(&self.conn, &mut authors, |row| row.get::<&str, String>("tag"))?;
        
        Ok(authors)
    }


    /// Gets a result of stored authores.
    /// TODO: USe FTS5 for improve the performance of this naive implementation.
    fn get_authors(
        &mut self,
        search: SearchConfig<ConfigInitialized>,
    ) -> Result<StoreResult<String>> {      
        let mut builder = QueryBuilder::new(&SELECT_AUTHORS_QUERY, search.as_ref());
        builder.use_where_clause(|txt| {
            let parts: Vec<String> = txt.split(' ').map(|s| format!("%{}%", s)).collect();
            let q = (0..parts.len()).map(|_| "name LIKE ?").collect::<Vec<&str>>().join(" AND ");
            (q, parts)
        });

        let mut authors: StoreResult<String> = StoreResult::default();
        builder.fetch(&self.conn, &mut authors, |row| row.get::<&str, String>("name"))?;
        
        Ok(authors)
    }

    fn get_book<T>(&mut self, id: T) -> Result<Book>
    where
        T: Borrow<i64>,
    {
        let query = format!("{} WHERE id = ?1", SELECT_BOOKS_QUERY);

        let book = self.conn.query_row(&query, [id.borrow()], |row| {
            Ok(map_sqlite_row_to_book!(&self.conn, row))
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

struct QueryBuilder<'a> {
    query: &'a str,
    text: &'a str,
    filter: Option<String>,
    skipped: &'a u64,
    sort_limit: String,
    search_params: Option<Vec<String>>,
    sort_params: Vec<&'a str>,
}

impl<'a> QueryBuilder<'a> {
    fn new(query: &'a str, config: &'a SearchConfig<ConfigInitialized>) -> Self {
        let mut sf = "".to_string();
        let mut sp: Vec<&'a str> = Vec::new();

        if let Some(sort) = config.get_sort_desc() {
            if !sort.is_empty() {
                sf.push_str("ORDER BY");
                for d in sort {
                    match d.1 {
                        SortOrder::Asc => sf.push_str(" ? ASC,"),
                        SortOrder::Desc => sf.push_str(" ? DESC,"),
                    }
                    sp.push(d.0.as_ref());
                }
                sf.pop();
                sf.push(' ');
            }
        }

        let mut skipped = &0u64;
        if let Some(l) = config.get_take() {
            match config.get_skip_page() {
                Some(s) if *s > 0 => {
                    sf.push_str(format!("LIMIT {}, {}", l, s).as_ref());
                    skipped = s;
                }
                _ => sf.push_str(format!("LIMIT {}", l).as_ref()),
            }
        }

        Self {
            query,
            text: config.get_text(),
            skipped,
            filter: None,
            search_params: None,
            sort_params: sp,
            sort_limit: sf,
        }
    }

    /// Use given function to construct the where clause.
    /// Uses first value of tuple to construct the where clause ` WHERE [first value of tuple]` and
    /// second argument is used as parameter in the given order.
    ///
    /// Example:
    /// ```rust
    /// builder.use_where_clause(|txt| ("name LIKE ? AND typ = ? ".to_string(), vec![txt.to_owned(), "employee".to_owned()]));
    /// ```
    fn use_where_clause<F>(&mut self, transform: F)
    where
        F: FnOnce(&str) -> (String, Vec<String>),
    {
        if !self.text.is_empty() {
            let fl = transform(self.text);
            if fl.0.is_empty() {
                return;
            }

            self.filter = Some(format!("WHERE {}", fl.0));
            self.search_params = Some(fl.1);
        }
    }

    /// Fetch queries the database with given connection and fills passed result struct.
    fn fetch<T, F>(&'a self, conn: &Connection, result: &mut StoreResult<T>, map: F) -> Result<()>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let mut query = self.query.to_owned();

        if let Some(f) = &self.filter {
            query.push(' ');
            query.push_str(f);
        }

        let count = if let Some(p) = &self.search_params {
            conn.query_row(
                format!("SELECT COUNT(*) FROM ({});", query).as_ref(),
                &p.iter()
                    .map(|s| s as &dyn rusqlite::ToSql)
                    .collect::<Vec<&dyn ToSql>>()[..],
                |row| row.get::<usize, u64>(0),
            )?
        } else {
            conn.query_row(
                format!("SELECT COUNT(*) FROM ({});", query).as_ref(),
                [],
                |row| row.get::<usize, u64>(0),
            )?
        };

        query.push(' ');
        query.push_str(&self.sort_limit);

        let mut all_params: Vec<&dyn ToSql> = Vec::new();
        if let Some(e) = &self.search_params {
            for (pos, _) in e.iter().enumerate() {
                all_params.push(&e[pos]);
            }
        }

        for (pos, _) in self.sort_params.iter().enumerate() {
            all_params.push(&self.sort_params[pos])
        }

        let params_iter = params_from_iter(all_params);
        let mut stmt = conn.prepare(&query)?;

        let res = stmt.query_map(params_iter, map)?;
        result.total = count;
        result.skipped = *self.skipped;

        for item in res {
            result.items.push(item?);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SqliteStore;
    use crate::books::models::{Book, BookDB, SortOrder};
    use crate::books::models::SearchConfig;
    use crate::sort_desc;
    
    use chrono::prelude::*;
    use chrono::Utc;
    
    use std::error::Error;

    type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

    #[test]
    fn myfn() -> Result {
        let mut db = SqliteStore::new("db_file")?;

        //let b = format!("{}", bla!("MEIN MACRO:", " -- ", "asdasd", "12", "SUPERDUPER", "and", "2345345", 11));

        println!(">>>>>>> {:?}", db.get_tags(SearchConfig::new("rel").use_sort(sort_desc!("tag", "asc")).build()));
        


        Ok(())
    }

    #[test]
    fn fetch_books() -> Result {
        let mut db = SqliteStore::new("db_file")?;

        let books = db.fetch_books(SearchConfig::new("").build())?;
        assert_eq!(books.total, 3);
        assert_eq!(books.skipped, 0);
        assert_eq!(books.items.len(), 3);

        let partial_books = db.fetch_books(SearchConfig::new("").use_skip_page(1).build())?;
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
