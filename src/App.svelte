<script lang="ts">
  import Greet from './lib/Greet.svelte';
  import init from './lang/i18n';
  import { useThemeStore } from '@/api';
  import { window } from '@tauri-apps/api';
  import { invoke } from '@tauri-apps/api/tauri';
  import { TauriEvent } from '@tauri-apps/api/event';

  window.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
    console.log('shutdown event received');
    await invoke('shutdown');
  });

  const { reloadTheme } = useThemeStore();

  let ready = false;
  (async () => {
    await init();
    await reloadTheme();
  })().then(() => (ready = true));
</script>

<main class="container">
  <h1>Welcome to Tauri!</h1>

  <div class="row">
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte" alt="Svelte Logo" />
    </a>
    <a href="#top">
      <div class="logo svelte app">
        <img src="/app_icon.png" alt="Bookshelf Logo" title="Einstellungen" />
      </div>
    </a>
  </div>

  <p>Click on the Tauri, Vite, and Svelte logos to learn more.</p>
  {#if ready}
    <div class="row">
      <Greet />
    </div>
  {/if}
</main>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  .logo.app > img {
    width: 96px;
    height: 96px;
  }
</style>
