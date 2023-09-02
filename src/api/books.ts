// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import { writable, readonly, type Readable } from "svelte/store";
import { invoke, type EventPayload } from "./utils";
import { listen, type Event } from "@tauri-apps/api/event";

const SET_CURRENT_SB = 'set_current_db';

const bookStore = writable<BookStore>({
  databases: [] as string[],
  current: undefined
}, (set, updater) => {  
  const unlisten = listen('book-manager-event', (event: Event<EventPayload<string|string[]>> ) => {
    console.log("received event", event);
  })

  updater(state => {
    console.log("state updated", state);
    return state;
  });
  

  return async () => (await unlisten)();   
}); 


const booksDB = writable<string[]>([]);
const currentDB = writable<string|undefined>();

// export const bookStore: BookStore = {
//   databases: readonly(booksDB),
//   current: readonly(currentDB)
// };


export interface BookStore {
  databases: string[];
  current: string|undefined;
}




export async function setCurrentDB(db: string): Promise<void> {
    // TODO: Show error notification
    await invoke(SET_CURRENT_SB, { args: {db} });
  }
  