<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';

  let name = '';
  let greetMsg = '';

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke('greet', { name });
  }

  async function create_book_db() {
    try {
      await invoke('create_book_db', { path: '' });
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
</div>
