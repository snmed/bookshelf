<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Copyright © 2024 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.

  interface SplitLayoutEvents {
    collapseChanged: boolean;
  }

  export let isOpen = true;
  export let autoOpen = true;

  let asideHovered = false;
  $: isCollapsed = !(isOpen || asideHovered);

  const dispatch = createEventDispatcher<SplitLayoutEvents>();

  const onCollapsedChanged = (collapsed: boolean) => {
    dispatch('collapseChanged', collapsed);
  };

  $: onCollapsedChanged(isCollapsed);
</script>

<div class="bs-split-layout w-full h-full {isCollapsed ? 'collapsed' : ''}">
  <div
    class="bs-split-layout-aside"
    on:mouseenter={() => (asideHovered = autoOpen && true)}
    on:mouseleave={() => (asideHovered = false)}
    role="presentation"
  >
    <slot name="aside" />
  </div>

  <div class="bs-split-layout-main">
    <slot />
  </div>
</div>

<style lang="postcss">
  .bs-split-layout {
    --bs-split-layout-collapsed: 4.25rem;
    --bs-split-layout-open: 12.5rem;
    --bs-split-layout-animation-ms: 500ms;

    display: grid;
    grid-template-columns: var(--bs-split-layout-open) auto;
    transition: grid-template-columns var(--bs-split-layout-animation-ms) ease;

    .bs-split-layout-main {
      padding: 1rem;
      overflow-y: auto;
    }
  }

  .bs-split-layout-aside:hover {
    grid-template-columns: var(--bs-split-layout-collapsed) auto;
  }

  .bs-split-layout.collapsed {
    grid-template-columns: var(--bs-split-layout-collapsed) auto;
  }
</style>
