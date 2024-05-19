// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import { locale } from 'svelte-i18n';
import { invoke } from './utils';
import {
  get,
  readonly,
  writable,
  type Readable,
  type Writable,
} from 'svelte/store';

const SET_LANGUAGE = 'set_lang';
const GET_HISTORY = 'get_history';
const REMOVE_HISTORY = 'remove_history';
const CURRENT_LANG = 'current_lang';
const CURRENT_THEME = 'current_theme';
const SET_THEME = 'set_theme';
const SET_MENU_EXPANDED = 'set_menu_expanded';
const GET_MENU_EXPANDED = 'get_menu_expanded';
const SET_MENU_AUTO_EXPAND = 'set_menu_auto_expand';
const GET_MENU_AUTO_EXPAND = 'get_menu_auto_expand';

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

/**
 * Sets menu expanded in the user settings.
 */
export async function setMenuExpanded(expanded: boolean): Promise<void> {
  await invoke(SET_MENU_EXPANDED, {
    args: { expanded },
  });
}

/**
 * Fetches current menu expanded value from the user settings.
 * @returns Current menu expanded value.
 */
export async function getMenuExpanded(): Promise<boolean> {
  return (await invoke<boolean>(GET_MENU_EXPANDED)).result ?? true;
}

export enum Themes {
  Dark = 'dark',
  Light = 'light',
}

export type Language = { id: string; label: string };
export type Theme = { theme: Themes; label: string };

export type SettingsApi = {
  lang: Readable<Language>;
  setLang: (l: Language) => Promise<void>;
  reloadLang: () => Promise<void>;
  availableLanguages: Language[];
  availableThemes: Theme[];
  theme: Readable<Themes>;
  setTheme: (t: Themes) => Promise<void>;
  reloadTheme: () => Promise<void>;
  menuAutoExpand: Readable<boolean>;
  reloadAutoExpand: () => Promise<void>;
  toggleMenuAutoExpand: () => Promise<void>;
};

const menuAutoExpandStore = writable(false);
const themeStore = writable(Themes.Dark);
const langStore: Writable<Language> = writable({
  id: 'en',
  label: 'labels.english',
});

const availableLanguages: Language[] = [
  { id: 'de', label: 'labels.german' },
  { id: 'en', label: 'labels.english' },
];

const availableThemes: Theme[] = [
  { theme: Themes.Dark, label: 'labels.dark-theme' },
  { theme: Themes.Light, label: 'labels.light-theme' },
];

export const useSettingsApi = () => {
  const updateLang = (l: Language) => {
    locale.set(l.id);
    langStore.set(l);
  };

  const setLang = async (l: Language): Promise<void> => {
    const res = await invoke(SET_LANGUAGE, { args: { lang: l.id } });
    if (!res.error) {
      updateLang(l);
    }
  };

  const reloadLang = async (): Promise<void> => {
    const l = await currentLang();
    const lang = availableLanguages.find((la) => la.id === l);
    if (lang) {
      updateLang(lang);
    }
  };

  const updateTheme = (theme: Themes) => {
    themeStore.set(theme);
    document.getElementsByTagName('html')[0].dataset.theme = theme;
  };

  const setTheme = async (theme: Themes) => {
    const res = await invoke(SET_THEME, { args: { theme } });
    if (!res.error) {
      updateTheme(theme);
    }
  };

  const reloadTheme = async () => {
    const res = await invoke<string>(CURRENT_THEME);
    updateTheme((res.result as Themes) ?? Themes.Dark);
  };

  const toggleMenuAutoExpand = async (): Promise<void> => {
    const current = get(menuAutoExpandStore);
    menuAutoExpandStore.set(!current);
    await invoke(SET_MENU_AUTO_EXPAND, {
      args: { autoExpand: !current },
    });
  };

  const reloadAutoExpand = async () => {
    const res = await invoke<boolean>(GET_MENU_AUTO_EXPAND);
    if (!res.error) {
      menuAutoExpandStore.set(res.result);
    }
  };

  return {
    lang: readonly(langStore),
    availableLanguages,
    availableThemes,
    theme: readonly(themeStore),
    menuAutoExpand: readonly(menuAutoExpandStore),
    toggleMenuAutoExpand,
    setTheme,
    setLang,
    reloadTheme,
    reloadLang,
    reloadAutoExpand,
  } as SettingsApi;
};
