<script lang="ts">
  // Copyright © 2023 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.

  import { useAppContext, type AppContext } from '@/contexts/app';
  import { writable, readonly } from 'svelte/store';
  import SplitLayout from '@/components/layouts/SplitLayout.svelte';
  import SideMenu from '@/components/layouts/SideMenu.svelte';
  import Greet from '@/lib/Greet.svelte';
  import { setMenuExpanded, getMenuExpanded } from '@/api';
  import { onMount } from 'svelte';

  const { setContext } = useAppContext();

  let isMenuOpen = true;
  let menuOpen = writable(isMenuOpen);
  const toggleMenu = async (show?: boolean) => {
    isMenuOpen = show ?? !isMenuOpen;
    menuOpen.set(isMenuOpen);
    await setMenuExpanded(isMenuOpen);
  };

  let collapsed = writable(isMenuOpen);
  const appContext: AppContext = {
    menuCollapsed: readonly(collapsed),
    menuOpen: readonly(menuOpen),
    toggleMenu,
  };

  function onCollapseChanged(e: CustomEvent<boolean>) {
    collapsed.set(e.detail);
  }

  setContext(appContext);

  onMount(async () => {
    await toggleMenu(await getMenuExpanded());
  });
</script>

<div class="bs-main-layout h-full w-full">
  <SplitLayout
    isOpen={isMenuOpen}
    autoOpen={false}
    on:collapseChanged={onCollapseChanged}
  >
    <SideMenu style="background: red" slot="aside"></SideMenu>

    <slot />
  </SplitLayout>
</div>

<style>
  .bs-main-layout {
    display: flex;
    flex-direction: row;
  }
</style>
