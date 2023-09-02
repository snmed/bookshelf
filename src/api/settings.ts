// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import { invoke } from "./utils";

const SET_LANGUAGE = 'set_lang';
const GET_HISTORY = 'get_history';
const REMOVE_HISTORY = 'remove_history';
const CURRENT_LANG = 'current_lang';

/**
 * Sets to language in user settings to `lang`.
 * @param lang Language to set
 */
export async function setCurrentLang(lang: string): Promise<void> {
  await invoke(SET_LANGUAGE, { args: {lang }});
}

/**
 * Fetches used database paths from the user settings.
 * @returns A list of used book database.
 */
export async function getHistory(): Promise<string[]> {
  return (await invoke<string[]>(GET_HISTORY)).result ?? [];
}

/**
 * Removes a path from the history.
 * @param path Path to remove.
 */
export async function removeHistory(path: string): Promise<void> {
  await invoke(REMOVE_HISTORY, { args: { path } });
}

/**
 * Fetches current language from the user settings.
 * @returns Current language.
 */
export async function currentLang(): Promise<string> {
    return (await invoke<string>(CURRENT_LANG)).result ?? '';
}
