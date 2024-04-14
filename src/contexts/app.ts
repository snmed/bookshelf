// Copyright © 2024 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import { getContext, setContext } from 'svelte';
import type { Readable } from 'svelte/store';
import { createContext } from './context-utils';

export type AppContext = {
  menuOpen: Readable<boolean>;
  toggleMenu: (show?: boolean) => void;
};

export const useAppContext = createContext<AppContext>('AppContextApi');
  

// export const useAppContext = () => {
//   let appContext =  getContext<AppContext>(appContextToken);
//     if(!appContext) {
//         appContext = new Proxy({}, {
//             get() {                
//                 throw new Error('[Context] context is not set and must be initialized first.');
//             },            
//         }) as AppContext;
//     }

//   return {
//     appContext,
//     setAppContext: (context: AppContext) =>
//       setContext(appContextToken, context),
//   };
// };
