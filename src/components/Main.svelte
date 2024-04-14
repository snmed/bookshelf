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

  const { setContext } = useAppContext();


  let isMenuOpen = true;

  let menuOpen = writable(isMenuOpen);
  const toggleMenu = (show?: boolean) => {
    isMenuOpen = show ?? !isMenuOpen
    menuOpen.set(isMenuOpen);
  }

  const appContext: AppContext = {
    menuOpen: readonly(menuOpen),
    toggleMenu
  };

  setContext(appContext);
</script>

<div class="bs-main-layout h-full w-full">
  <SplitLayout isOpen={isMenuOpen} autoOpen={true}>
    <SideMenu slot="aside"></SideMenu>

    <Greet></Greet>
  </SplitLayout>
</div>

<style>
  .bs-main-layout {
    display: flex;
    flex-direction: row;
  }
</style>
