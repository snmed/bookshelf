// Copyright © 2024 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import type { Readable } from 'svelte/store';
import { createContext } from './context-utils';

export type AppContext = {
  menuOpen: Readable<boolean>;
  menuCollapsed: Readable<boolean>;
  toggleMenu: (show?: boolean) => Promise<void>;
};

export const useAppContext = createContext<AppContext>('AppContextApi');
