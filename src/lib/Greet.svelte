<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen } from '@tauri-apps/api/event'
  import { _, locale, locales } from 'svelte-i18n';
  import { getHistory, setCurrentDB } from '@/api';
  import { onDestroy } from 'svelte';
  let name = '';
  let greetMsg = '';

  const unlisten = listen('book-manager-event', (p) => {
    console.log("received event", p)
  });

  onDestroy(async () => (await unlisten)());

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    //greetMsg = await invoke('greet', { name });
    try {
      await setCurrentDB(name);  
    } catch (error) {
        console.error("failed to set current DB", error);
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
  <form class="row" on:submit|preventDefault={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <button on:click={create_book_db}>Create DB</button>
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
</div>
