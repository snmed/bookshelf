<script lang="ts">
  import { useSettingsApi, type Language, type Theme } from '@/api/settings';
  // Copyright © 2024 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.
  import Page from '@/lib/Page.svelte';
  import { onDestroy } from 'svelte';
  import { _ } from 'svelte-i18n';
  import type { Unsubscriber } from 'svelte/motion';

  const subscriptions: Unsubscriber[] = [];
  const {
    availableLanguages,
    availableThemes,
    lang,
    theme,
    setLang,
    setTheme,
    toggleMenuAutoExpand,
    menuAutoExpand,
  } = useSettingsApi();

  let selectedLang: Language;
  subscriptions.push(lang.subscribe((l) => (selectedLang = l)));

  let selectedTheme: Theme;
  subscriptions.push(
    theme.subscribe(
      (t) => (selectedTheme = availableThemes.find((th) => th.theme === t)),
    ),
  );

  onDestroy(() => subscriptions.forEach((s) => s()));
</script>

<Page title={$_('labels.settings')}>
  <label class="form-control w-full max-w-sm">
    <div class="label">
      <span class="label-text">{$_('labels.language')}</span>
    </div>
    <select
      class="select select-bordered"
      bind:value={selectedLang}
      on:change={() => setLang(selectedLang)}
    >
      {#each availableLanguages as l}
        <option value={l}>{$_(l.label)}</option>
      {/each}
    </select>
  </label>

  <label class="form-control w-full max-w-sm mt-5">
    <div class="label">
      <span class="label-text">{$_('labels.theme')}</span>
    </div>
    <select
      class="select select-bordered"
      bind:value={selectedTheme}
      on:change={() => setTheme(selectedTheme.theme)}
    >
      {#each availableThemes as t}
        <option value={t}>{$_(t.label)}</option>
      {/each}
    </select>
  </label>

  <div class="form-control w-full max-w-sm mt-5">
    <label class="label cursor-pointer">
      <span class="label-text">{$_('labels.menu-auto-open')}</span>
      <input
        type="checkbox"
        class="toggle"
        checked={$menuAutoExpand}
        on:change={() => toggleMenuAutoExpand()}
      />
    </label>
  </div>
</Page>

<style>
</style>
