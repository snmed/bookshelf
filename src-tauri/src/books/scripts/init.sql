/*
 * Script:      init.sql
 * Description: Setup bookshelf schema.
 *
 * Author:      Sandro Dallo
 * Date:        06.08.2023
 */

PRAGMA foreign_keys = ON;

-- Primary books table
CREATE TABLE IF NOT EXISTS books (
   id INTEGER PRIMARY KEY AUTOINCREMENT,
   cover_img BLOB,
   description TEXT,
   isbn TEXT NOT NULL,
   lang TEXT NOT NULL,
   title TEXT NOT NULL,
   sub_title TEXT,
   publisher TEXT,
   created INTEGER NOT NULL,
   updated INTEGER NOT NULL
);

-- Authors
CREATE TABLE IF NOT EXISTS authors (
   name TEXT NOT NULL,
   book_id INTEGER NOT NULL,
   FOREING KEY book_id REFERENCES books(id) ON DELETE CASCADE
);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
   value TEXT NOT NULL,
   book_id INTEGER NOT NULL,
   FOREING KEY book_id REFERENCES books(id) ON DELETE CASCADE
);
