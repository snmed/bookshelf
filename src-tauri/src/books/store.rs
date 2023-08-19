// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use std::ops::Add;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{named_params, params, Connection, ToSql};
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

// #[cfg(debug_assertions)]
// fn create_sqlite_connection(_: &str) -> Result<Connection> {
//     Ok(Connection::open_in_memory()?)
// }

// #[cfg(not(debug_assertions))]
fn create_sqlite_connection(db_file: &str) -> Result<Connection> {
    Ok(Connection::open(db_file)?)
}

#[derive(Debug)]
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn new(db_file: &str) -> Result<Self> {
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

    fn delete_book_by_id(&mut self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM books WHERE id = ?", [id])?;
        Ok(())
    }

    fn fetch_books(
        &mut self,
        search: SearchConfig<ConfigInitialized>,
    ) -> Result<StoreResult<Book>> {
        // This is quite naive implementation, use FTS5 to improve search performance.
        let query = if search.get_text() != "" {
            SELECT_BOOKS_QUERY.to_owned().add(
                r#" WHERE id IN (
                SELECT DISTINCT B.id
                FROM books as B
                    LEFT JOIN authors AS A ON A.book_id = B.id
                    LEFT JOIN tags AS T ON T.book_id = B.id
                WHERE B.title LIKE ?
                    OR B.sub_title LIKE ?
                    OR B.publisher LIKE ?
                    OR B.isbn LIKE ?
                    OR B.description LIKE ?
                    OR A.name LIKE ?
                    OR T.tag LIKE ?
            );"#,
            )
        } else {
            SELECT_BOOKS_QUERY.to_owned()
        };

        let mut builder = QueryBuilder::new(&query, &search);
        let txt = format!("%{}%", search.get_text());
        if search.get_text() != "" {
            builder.use_params(vec![&txt, &txt, &txt, &txt, &txt, &txt, &txt, &txt, &txt])?;
        }

        let mut books: StoreResult<Book> = StoreResult::default();
        builder.fetch(&self.conn, &mut books, |row| {
            Ok(map_sqlite_row_to_book!(&self.conn, row))
        })?;

        Ok(books)
    }

    /// Gets a result of stored tags.
    /// TODO: USe FTS5 for improve the performance of this naive implementation.
    fn get_tags(&mut self, search: SearchConfig<ConfigInitialized>) -> Result<StoreResult<String>> {
        let mut builder = QueryBuilder::new(SELECT_TAGS_QUERY, search.as_ref());
        builder.use_where_clause(|txt| ("tag LIKE ?".to_owned(), vec![format!("%{}%", txt)]))?;

        let mut authors: StoreResult<String> = StoreResult::default();
        builder.fetch(&self.conn, &mut authors, |row| {
            row.get::<&str, String>("tag")
        })?;

        Ok(authors)
    }

    /// Gets a result of stored authores.
    /// TODO: USe FTS5 for improve the performance of this naive implementation.
    fn get_authors(
        &mut self,
        search: SearchConfig<ConfigInitialized>,
    ) -> Result<StoreResult<String>> {
        let mut builder = QueryBuilder::new(SELECT_AUTHORS_QUERY, search.as_ref());
        builder.use_where_clause(|txt| {
            let parts: Vec<String> = txt.split(' ').map(|s| format!("%{}%", s)).collect();
            let q = (0..parts.len())
                .map(|_| "name LIKE ?")
                .collect::<Vec<&str>>()
                .join(" AND ");
            (q, parts)
        })?;

        let mut authors: StoreResult<String> = StoreResult::default();
        builder.fetch(&self.conn, &mut authors, |row| {
            row.get::<&str, String>("name")
        })?;

        Ok(authors)
    }

    fn get_book(&mut self, id: i64) -> Result<Book> {
        let query = format!("{} WHERE id = ?1", SELECT_BOOKS_QUERY);

        let book = self.conn.query_row(&query, [id], |row| {
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
    params: Option<Vec<&'a dyn ToSql>>,
}

impl<'a> QueryBuilder<'a> {
    fn new(query: &'a str, config: &'a SearchConfig<ConfigInitialized>) -> Self {
        let mut sf = "".to_string();

        if let Some(sort) = config.get_sort_desc() {
            if !sort.is_empty() {
                sf.push_str("ORDER BY");
                for d in sort {
                    match d.1 {
                        SortOrder::Asc => sf.push_str(format!(" {} ASC,", d.0).as_ref()),
                        SortOrder::Desc => sf.push_str(format!(" {} DESC,", d.0).as_ref()),
                    }
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
            sort_limit: sf,
            params: None,
        }
    }

    /// Use given function to construct the where clause.
    /// Uses first value of tuple to construct the where clause ` WHERE [first value of tuple]` and
    /// second argument is used as parameter in the given order.
    /// Use either `use_params` or `use_where_clause`.
    ///
    /// Example:
    /// ```rust
    /// builder.use_where_clause(|txt| ("name LIKE ? AND typ = ? ".to_string(), vec![txt.to_owned(), "employee".to_owned()]));
    /// ```
    fn use_where_clause<F>(&mut self, transform: F) -> Result<()>
    where
        F: FnOnce(&str) -> (String, Vec<String>),
    {
        if self.params.is_some() {
            return Err(BookError::Generic(
                "Either use 'use_where_clause' or 'use_params'".to_owned(),
            ));
        }

        if !self.text.is_empty() {
            let fl = transform(self.text);
            if fl.0.is_empty() {
                return Ok(());
            }

            self.filter = Some(format!("WHERE {}", fl.0));
            self.search_params = Some(fl.1);
        }

        Ok(())
    }

    /// Use given parameters for constructed query. Use either `use_params` or `use_where_clause`.
    fn use_params(&mut self, params: Vec<&'a dyn ToSql>) -> Result<()> {
        if self.filter.is_some() {
            return Err(BookError::Generic(
                "Either use 'use_where_clause' or 'use_params'".to_owned(),
            ));
        }

        self.params = Some(params);
        Ok(())
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

        let mut clause_params: Vec<&dyn ToSql> = Vec::new();
        let all_params = {
            if let Some(e) = &self.search_params {
                for (pos, _) in e.iter().enumerate() {
                    clause_params.push(&e[pos]);
                }
                &clause_params[..]
            } else if let Some(p) = &self.params {
                &p[..]
            } else {
                params![]
            }
        };

        let mut stmt = conn.prepare(&query)?;

        let res = stmt.query_map(all_params, map)?;
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
    use crate::books::models::SearchConfig;
    use crate::books::models::{Book, BookDB};
    use chrono::prelude::*;
    use chrono::Utc;
    use std::error::Error;

    type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

    macro_rules! cmp_book {
        (@Vec $a:expr, $b:expr, $comment:literal) => {{
            let mut aa: Vec<String> = $a.clone();
            let mut bb: Vec<String> = $b.clone();
            aa.sort();
            bb.sort();
            assert_eq!(aa, bb);
        }};
        ($a:expr, $b:expr) => {
            assert_eq!($a.id, $b.id, "Book id mismatch");
            assert_eq!($a.cover_img, $b.cover_img, "Cover image mismatch");
            assert_eq!($a.description, $b.description, "Description mismatch");
            assert_eq!($a.isbn, $b.isbn, "ISBN mismatch");
            assert_eq!($a.lang, $b.lang, "Language mismatch");
            assert_eq!($a.title, $b.title, "Title mismatch");
            assert_eq!($a.sub_title, $b.sub_title, "Sub title mismatch");
            assert_eq!($a.publisher, $b.publisher, "Publisher mismatch");
            assert_eq!($a.publish_date, $b.publish_date, "Publisher date mismatch");

            cmp_book!(@Vec $a.authors, $b.authors, "Authors mismatched");

            assert_eq!($a.tags.is_some(), $b.tags.is_some(), "Tags mismatch");
            if $a.tags.is_some() {
                cmp_book!(@Vec $a.tags.as_ref().unwrap(), $b.tags.as_ref().unwrap(), "Tags mismatched");
            }

        };
    }

    macro_rules! cmp_vec_books {
        ($testee:expr, $expected:expr) => {
            assert!($testee.len() == $expected.len(), "Books count mismatch");

            $expected.sort_by(|a, b| a.id.cmp(&b.id));
            $testee.sort_by(|a, b| a.id.cmp(&b.id));

            for (pos, item) in $testee.iter().enumerate() {
                cmp_book!(item, $expected[pos]);
            }
        };
    }

    #[test]
    fn fetch_books() -> Result {
        let mut db = SqliteStore::new("db_file")?;

        let mut books = db.fetch_books(SearchConfig::new("").build())?;
        assert_eq!(books.total, 3);
        assert_eq!(books.skipped, 0);
        assert_eq!(books.items.len(), 3);

        cmp_vec_books!(
            books.items,
            vec![Book {
                authors: vec!["David Lagercrantz".to_owned()],
                cover_img: None,
                description: Some("Lisbeth Salander is an unstoppable force!".to_owned()),
                isbn: "9780857056429".to_owned(),
                lang: "EN".to_owned(),
                tags: Some(vec!["Thriller".to_owned(), "Suspense".to_owned()]),
                title: "The Girl Who Takes an Eye for an Eye".to_owned(),
                sub_title: None,
                publisher: Some("McLehose Press".to_owned()),
                publish_date: Some(Utc.timestamp_opt(1483523713, 0).unwrap()),
                id: 1,

                ..Default::default()
            }, Book {
                authors: vec!["Jochen Schiller".to_owned()],
                cover_img: None,
                description: Some("Explains mobile communications in details.".to_owned()),
                isbn: "9780321123817".to_owned(),
                lang: "EN".to_owned(),
                tags: Some(vec!["Data Transmission Systems".to_owned(), "Wireless".to_owned(), "Communications".to_owned()]),
                title: "Mobile Communications".to_owned(),
                sub_title: Some("Second Edition".to_owned()),
                publisher: Some("Addison Wesley".to_owned()),
                publish_date: Some(Utc.timestamp_opt(1062150913, 0).unwrap()),
                id: 2,

                ..Default::default()
            }, Book {
                authors: vec!["Richard Dawkins".to_owned()],
                cover_img: None,
                description: Some("Richard Dawkins provozierendes Buch beseitigt jeden Zweifel an Darwins Theorie.".to_owned()),
                isbn: "9783550087653".to_owned(),
                lang: "DE".to_owned(),
                tags: Some(vec!["Wissenschaft".to_owned(), "Biologie".to_owned(), "Religion".to_owned()]),
                title: "Es gibt keine Schöpfung".to_owned(),
                sub_title: None,
                publisher: Some("Ullstein Verlag".to_owned()),
                publish_date: Some(Utc.timestamp_opt(1283075713, 0).unwrap()),
                id: 3,

                ..Default::default()
            }]
        );

        let mut partial_books =
            db.fetch_books(SearchConfig::new("").use_take(2).use_skip_page(1).build())?;
        assert_eq!(partial_books.total, 3);
        assert_eq!(partial_books.skipped, 1);
        assert_eq!(partial_books.items.len(), 1);

        cmp_vec_books!(partial_books.items, vec![Book {
            authors: vec!["Richard Dawkins".to_owned()],
            cover_img: None,
            description: Some("Richard Dawkins provozierendes Buch beseitigt jeden Zweifel an Darwins Theorie.".to_owned()),
            isbn: "9783550087653".to_owned(),
            lang: "DE".to_owned(),
            tags: Some(vec!["Wissenschaft".to_owned(), "Biologie".to_owned(), "Religion".to_owned()]),
            title: "Es gibt keine Schöpfung".to_owned(),
            sub_title: None,
            publisher: Some("Ullstein Verlag".to_owned()),
            publish_date: Some(Utc.timestamp_opt(1283075713, 0).unwrap()),
            id: 3,

            ..Default::default()
        }]);

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
