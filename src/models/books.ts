// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

export class Book {
  id: number = 0;
  cover_img: string | null = null;
  description: string | null = null;
  isbn: string = '';
  lang: string = '';
  publisher: string | null = null;
  title: string = '';
  sub_title: string | null = null;
  tags: string[] | null = null;
  authors: string[] = [];

  #publish_date: string | null = null;
  get publish_date(): Date | null {
    if (!this.#publish_date) return null;
    return new Date(this.#publish_date);
  }

  set publish_date(date: string|Date) {
    if (date instanceof Date) {
        this.#publish_date = date.toISOString();
        return;
    }
    this.#publish_date = date ?? null;
  }

  #created: string = '';
  get created(): Date | null {
    if (this.#created === '') return null;
    return new Date(this.#created);
  }

  private set created(date: string) {
    this.#created = date ?? '';
  }

  #updated: string = '';
  get updated(): Date | null {
    if (this.#updated === '') return null;
    return new Date(this.#updated);
  }

  private set updated(date: string) {
    this.#updated = date ?? '';
  }

  constructor(book?: Partial<Book>) {
    if (book) Object.assign(this, book);
  }

  toJSON() {
    return {
        ...this,
        publish_date: this.#publish_date,
        created: this.#created,
        updated: this.#updated
    };
  }
}
