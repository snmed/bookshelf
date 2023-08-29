<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { _, locale } from 'svelte-i18n';


  let name = '';
  let greetMsg = '';

  async function greet() {

    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke('greet', { name });
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

</div>
