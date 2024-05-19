<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { Themes, useSettingsApi } from '@/api';
  import { _, locale, locales } from 'svelte-i18n';
  import {
    getHistory,
    setCurrentDB,
    getBook,
    currentBookDB,
    openBookDB,
  } from '@/api';
  import { onDestroy } from 'svelte';
  import Icon from '@/components/Icon.svelte';
  import { useAppContext } from '@/contexts/app';

  const themeStore = useSettingsApi();

  const { context } = useAppContext();

  let name = '';
  let greetMsg = '';

  let current: Themes = Themes.Dark;
  const unsubscribe = themeStore.theme.subscribe((t) => {
    console.log(`>>>>>>>>>>>>>>>>>> SUBSCRIPTION`, t);
    current = t;
  });

  onDestroy(unsubscribe);

  const switchTheme = async () => {
    themeStore.setTheme(current == Themes.Dark ? Themes.Light : Themes.Dark);
  };

  let currentTheme = '';
  $: switch (current) {
    case Themes.Dark:
      currentTheme = 'Dark';
      break;
    case Themes.Light:
      currentTheme = 'Light';
      break;
    default:
      currentTheme = '';
  }

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    //greetMsg = await invoke('greet', { name });
    try {
      await getBook(parseInt(name));
    } catch (error) {
      console.error('failed to set current DB', error);
    }
  }

  async function create_book_db() {
    try {
      console.log(await invoke('create_book_db'));
    } catch (e: unknown) {
      console.log(`Error: ${JSON.stringify(e)}`);
    }
  }
</script>

<div>
  <p>{$currentBookDB}</p>
  <p>
    {#each $openBookDB as db, i}
      <div>{db} - {i}</div>
    {/each}
  </p>
  <form class="row" on:submit|preventDefault={greet}>
    <input
      class="input input-bordered w-full max-w-xs"
      id="greet-input"
      placeholder="Enter a name..."
      bind:value={name}
    />
    <button class="btn" type="submit">Greet</button>
  </form>
  <button class="btn" on:click={create_book_db}>Create DB</button>
  <p>{greetMsg}</p>
  <p>{$locale}</p>
  <p>{$_('labels.settings')}</p>
  <p>
    {$_('messages.books-import-success', { values: { book: 'Star Wars' } })}
  </p>
  <button on:click={() => locale.set('de')}>Set German</button>
  {#each $locales as l}
    <p>{l}</p>
  {/each}

  {#await getHistory()}
    Loading....
  {:then data}
    {#each data as name, i}
      <div>{name} - {i}</div>
    {/each}
  {/await}
  <button class="btn btn-secondary" on:click={() => context.toggleMenu()}
    >Switch Menu</button
  >
  <p>Current Theme: {current}</p>
  <p>Current Theme: {currentTheme}</p>
  <Icon name="cog-solid" class="super"></Icon>
  <select class="select select-bordered w-full max-w-xs">
    <option disabled selected>Who shot first?</option>
    <option>Han Solo</option>
    <option>Greedo</option>
  </select>
  <button class="btn btn-secondary" on:click={switchTheme}>Switch Theme</button>
</div>

<style>
  :global(.super) {
    width: 256px;
    height: 256px;
  }
</style>
