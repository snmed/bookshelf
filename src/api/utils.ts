// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

import { invoke as tauriInvoke, type InvokeArgs } from '@tauri-apps/api/tauri';

export interface ApiError {
  error: string;
  code: number;
}

export interface InvokeOptions {
  args?: InvokeArgs;
  showUserNotificaton?: boolean;
  rethrow?: boolean;
}

export interface Result<T> {
  result?: T;
  error?: ApiError;
}


export interface EventPayload<T> {
  content: T,
  type: string
}


export async function invoke<T = void>(
  cmd: string,
  opts?: InvokeOptions,
): Promise<Result<T>> {
  try {
    return { result: await tauriInvoke<T>(cmd, opts?.args) };
  } catch (error: unknown) {
    const apiError = error as ApiError;
    console.error(
      `error while calling bookshelf backend: '${apiError.error}' Code: ${apiError.code}`,
    );
    if (!!opts?.showUserNotificaton) {
      // TODO: Show user notification with some ui serivce
      alert(`${apiError.error} -> ${apiError.code}`);
    }

    if (!!opts?.rethrow) throw apiError;

    return { error: apiError };
  }
}
