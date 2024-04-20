<script lang="ts">
  import '@/styles.css';
  import init from '@/lang/i18n';
  import Main from '@/components/Main.svelte';
  import { useThemeStore } from '@/api';
  import { window } from '@tauri-apps/api';
  import { invoke } from '@tauri-apps/api/tauri';
  import { TauriEvent } from '@tauri-apps/api/event';

  window.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
    console.log('shutdown event received');
    await invoke('shutdown');
  });

  const { reloadTheme } = useThemeStore();

  const ready = async () => {
    await init();
    await reloadTheme();
  };
</script>

{#await ready()}
  Loading ...
{:then _}
  <Main>
    <slot />
  </Main>
{/await}
