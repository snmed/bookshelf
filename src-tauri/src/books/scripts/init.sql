/*
 * Script:      init.sql
 * Description: Setup bookshelf schema.
 *
 * Author:      Sandro Dallo
 * Date:        06.08.2023
 */

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