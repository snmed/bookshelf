// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import { resolveResource } from '@tauri-apps/api/path';
import { readTextFile } from '@tauri-apps/api/fs';
import { register, init, waitLocale } from 'svelte-i18n';

let messages: { [key: string]: any } | undefined;
async function loadMessageFile() {
  if (!messages) {
    const resourcePath = await resolveResource('resources/messages.json');
    messages = JSON.parse(await readTextFile(resourcePath));
  }
  return messages;
}

register('en', async () => (await loadMessageFile())!['en']);
register('de', async () => (await loadMessageFile())!['de']);

export default async () => {
  await init({
    fallbackLocale: 'en',
    initialLocale: 'en',
  });
};
