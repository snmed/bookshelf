// Copyright © 2024 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.
import { getContext, setContext } from 'svelte';

type ContextFunc<TContext> = () => Context<TContext>;
type Context<TContext> = {
  context: TContext;
  setContext: (ctx: TContext) => void;
};

export const createContext = <TContext>(
  contextKey: string
): ContextFunc<TContext> => {
  
  return () => {
    let context = getContext<TContext>(contextKey);
    if (!context) {
      context = new Proxy(
        {},
        {
          get() {
            throw new Error(
              `[Context] context ${contextKey} is not set and must be initialized first.`,
            );
          },
        },
      ) as TContext;
    }

    return {
      context,
      setContext: (context: TContext) =>
        setContext(contextKey, context)
    };
  }; 
};
