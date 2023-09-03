// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import { writable, readonly, type Readable } from 'svelte/store';
import { invoke, type EventPayload } from './utils';
import { listen, type Event } from '@tauri-apps/api/event';
import { Book } from '@/models/books';

const SET_CURRENT_DB = 'set_current_db';
const GET_BOOK = 'get_book';
const BOOK_MANAGER_EVENT = 'book-manager-event';

const bookDatabases = writable<string[]>([]);
const currentDatabase = writable<string | undefined>();

export const openBookDB = readonly(bookDatabases);
export const currentBookDB = readonly(currentDatabase);

listen(BOOK_MANAGER_EVENT, (event: Event<EventPayload<string | string[]>>) => {
  console.debug(
    `received book manager event ${event.payload.type}`,
    event.payload.content,
  );
  switch (event.payload.type) {
    case 'CurrentDBChanged':
      currentDatabase.update((db) => {
        if (
          db !== event.payload.content &&
          typeof event.payload.content === 'string'
        ) {
          return event.payload.content;
        }
        return db;
      });
      break;
    case 'OpenDBChanged':
      if (Array.isArray(event.payload.content)) {
        bookDatabases.set(event.payload.content);
      }
      break;
    default:
      break;
  }
});

export async function setCurrentDB(db: string): Promise<void> {
  // TODO: Show error notification
  await invoke(SET_CURRENT_DB, { args: { db } });
}

export async function getBook(id: number): Promise<void> {
  const result = await invoke<Partial<Book>>(GET_BOOK, { args: { id } });
  const book = new Book(result.result)
  console.log('Received book', JSON.parse(JSON.stringify(book)));
}
