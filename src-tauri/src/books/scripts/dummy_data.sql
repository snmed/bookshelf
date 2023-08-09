/*
 * Script:      dummy_data.sql
 * Description: Inserts some dummy data into the database.
 *
 * Author:      Sandro Dallo
 * Date:        08.08.2023
 */

-- First Development Book
INSERT INTO books (
        description,
        isbn,
        lang,
        title,
        publisher,
        publish_date,
        created,
        updated
    )
VALUES (
        'Lisbeth Salander is an unstoppable force!',
        '9780857056429',
        'EN',
        'The Girl Who Takes an Eye for an Eye',
        'McLehose Press',
        1483523713,
        unixepoch(),
        unixepoch()
    );
    
INSERT INTO authors (name, book_id)
SELECT 'David Lagercrantz',
    (
        select MAX(id)
        from books
    );

INSERT INTO tags (tag, book_id)
SELECT 'Thriller',
    (
        select MAX(id)
        from books
    )
UNION
SELECT 'Suspense',
    (
        select MAX(id)
        from books
    );

-- Second Development Book
INSERT INTO books (
        description,
        isbn,
        lang,
        title,
        sub_title,
        publisher,
        publish_date,
        created,
        updated
    )
VALUES (
        'Explains mobile communications in details.',
        '9780321123817',
        'EN',
        'Mobile Communications',
        'Second Edition',
        'Addison Wesley',
        1062150913,
        unixepoch(),
        unixepoch()
    );
    
INSERT INTO authors (name, book_id)
SELECT 'Jochen Schiller',
    (
        select MAX(id)
        from books
    );

INSERT INTO tags (tag, book_id)
SELECT 'Data Transmission Systems',
    (
        select MAX(id)
        from books
    )
UNION
SELECT 'Communications',
    (
        select MAX(id)
        from books
    )
UNION
SELECT 'Wireless',
    (
        select MAX(id)
        from books
    );

-- Third Development Book
INSERT INTO books (
        description,
        isbn,
        lang,
        title,
        publisher,
        publish_date,
        created,
        updated
    )
VALUES (
        'Richard Dawkins provozierendes Buch beseitigt jeden Zweifel an Darwins Theorie.',
        '9783550087653',
        'DE',
        'Es gibt keine Schöpfung',
        'Ullstein Verlag',
        1283075713,
        unixepoch(),
        unixepoch()
    );
    
INSERT INTO authors (name, book_id)
SELECT 'Richard Dawkins',
    (
        select MAX(id)
        from books
    );

INSERT INTO tags (tag, book_id)
SELECT 'Wissenschaft',
    (
        select MAX(id)
        from books
    )
UNION
SELECT 'Biologie',
    (
        select MAX(id)
        from books
    )
UNION
SELECT 'Religion',
    (
        select MAX(id)
        from books
    );