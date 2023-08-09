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
   cover_img TEXT,
   description TEXT,
   isbn TEXT NOT NULL,
   lang TEXT NOT NULL,
   title TEXT NOT NULL,
   sub_title TEXT,
   publisher TEXT,
   publish_date INTEGER,
   created INTEGER NOT NULL,
   updated INTEGER NOT NULL
);

-- Authors
CREATE TABLE IF NOT EXISTS authors (
   name TEXT NOT NULL,
   book_id INTEGER NOT NULL,
   CONSTRAINT FK_books_authors FOREIGN KEY(book_id) REFERENCES books(id) ON DELETE CASCADE
);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
   tag TEXT NOT NULL,
   book_id INTEGER NOT NULL,
   CONSTRAINT FK_books_tags FOREIGN KEY(book_id) REFERENCES books(id) ON DELETE CASCADE
);
