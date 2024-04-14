<script lang="ts">
  // Copyright © 2024 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.
  
  export let isOpen = true;
  export let autoOpen = true;

  let asideHovered = false;
  $: isCollapsed = !(isOpen || asideHovered);
</script>

<div class="bs-split-layout w-full h-full {isCollapsed ? 'collapsed' : ''}">
  <div
    class="bs-split-layout-aside"
    on:mouseenter={() => (asideHovered = autoOpen && true)}
    on:mouseleave={() => (asideHovered = false)}
    role="presentation"
  >
    <slot name="aside"></slot>
  </div>

  <div class="bs-split-layout-main">
    <slot></slot>
  </div>
</div>

<style lang="postcss">
  .bs-split-layout {
    --bs-split-layout-collapsed: 68px;
    --bs-split-layout-open: 200px;
    --bs-split-layout-animation-ms: 500ms;

    display: grid;
    grid-template-columns: var(--bs-split-layout-open) auto;
    transition: grid-template-columns var(--bs-split-layout-animation-ms) ease;


    .bs-split-layout-main {
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
