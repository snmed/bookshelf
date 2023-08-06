// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use super::models::{Book, BookDB, BookError, SearchConfig, StoreResult, Tag};

pub struct SqliteStore {
}

impl BookDB for SqliteStore {
    fn add_book(&self, book: Book) -> Result<Book, BookError> {
        todo!()
    }

    fn update_book(&self, book: &mut Book) -> Result<(), BookError> {
        todo!()
    }

    fn delete_book(&self, book: &Book) -> Result<(), BookError> {
        todo!()
    }

    fn delete_book_by_id(&self, id: i64) -> Result<(), BookError> {
        todo!()
    }

    fn fetch_books(&self, search: &SearchConfig) -> Result<StoreResult<Book>, BookError> {
        todo!()
    }

    fn get_tags(&self, pattern: &str) -> Result<StoreResult<Tag>, BookError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rusqlite::Connection;
    use rusqlite_migration::{Migrations, M};


    #[test]
    fn db_test_fn() {

        let initSQL = include_str!("scripts/init.sql");
        println!("INIT SCRIPT {}", initSQL);
        let migrations = Migrations::new(vec![
            M::up(initSQL)
        ]);

        let mut con = Connection::open("./bla.db").unwrap();
        match migrations.to_latest(&mut con) {
            Ok(_) => println!("Migrations done"),
            Err(e) => println!("An error occurred {}", e)
        }
        
        con.close();
    }

}