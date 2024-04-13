<script context="module" lang="ts">
  export type myNewType = { name: string };
</script>

<script lang="ts">
  // Copyright © 2024 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.
  import Greet from '@/lib/Greet.svelte';
  import { createEventDispatcher } from 'svelte';

  export let isOpen = true;
  export let autoOpen = true;

  function toggle() {
    isOpen = !isOpen;
  }

  let asideHovered = false;

  $: isCollapsed = !(isOpen || asideHovered);
</script>

<div class="bs-split-layout w-full h-full {isCollapsed ? 'collapsed' : ''}">
  <div
    class="bs-split-layout-aside bg-base-200"
    on:mouseenter={() => (asideHovered = autoOpen && true)}
    on:mouseleave={() => (asideHovered = false)}
    role="presentation"
  >
    <ul class="menu bg-base-200">
      <li>
        <a href="#top">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
            /></svg
          >
        </a>
      </li>

      <li>
        <a href="#top" class="">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
            /></svg
          >
        </a>
      </li>
    </ul>
  </div>

  <div class="bs-split-layout-main">
    <p>Main Content</p>
    <button class="btn" on:click={toggle}>Greet</button>
    <Greet></Greet>
  </div>
</div>

<style>
  .bs-split-layout {
    --bs-split-layout-collapsed: 68px;
    --bs-split-layout-open: 200px;
    --bs-split-layout-animation-ms: 500ms;

    display: grid;
    grid-template-columns: var(--bs-split-layout-open) auto;
    transition: grid-template-columns var(--bs-split-layout-animation-ms) ease;
  }

  .bs-split-layout-aside:hover {
    grid-template-columns: var(--bs-split-layout-collapsed) auto;
  }

  .bs-split-layout.collapsed {
    grid-template-columns: var(--bs-split-layout-collapsed) auto;
    background-color: blue;
  }
</style>
