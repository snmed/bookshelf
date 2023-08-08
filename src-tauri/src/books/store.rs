// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use super::models::{Book, BookDB, BookError, SearchConfig, StoreResult};


/// Opens or creates a new books database and returns it.
pub fn open_connection(db_file: &str) -> Result<Connection, BookError> {
    // Add all required sql scripts to the migrator
    let mut scripts = vec![M::up(include_str!("scripts/init.sql"))];

    // Add only for debug mode dummy data
    if cfg!(debug_assertions) {
        scripts.push(M::up(include_str!("scripts/dummy_data.sql")));
    }

    let mut conn = create_connection(db_file)?;
    let migrations = Migrations::new(scripts);

    migrations.to_latest(&mut conn)?;    

    Ok(conn)
}

#[cfg(debug_assertions)]
fn create_connection(_: &str) -> Result<Connection, BookError> {
    Ok(Connection::open_in_memory()?)
}

#[cfg(not(debug_assertions))]
fn create_connection(db_file: &str) -> Result<Connection, BookError> {
    Ok(Connection::open(db_file)?)
}

pub struct SqliteStore {}

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

    fn get_tags(&self, pattern: &str) -> Result<StoreResult<String>, BookError> {
        todo!()
    }

    fn get_authors(&self, search: &SearchConfig) -> Result<StoreResult<String>, BookError> {
        todo!()
    }
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

#[cfg(test)]
mod tests {

    use rusqlite::Connection;
    use rusqlite_migration::{Migrations, M};

    use crate::books::models::BookError;

    use super::open_connection;

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

        let conn = open_connection("blabla.db")?;
        println!(">>>>>>> {:?}", conn);



        Ok(())

        

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
