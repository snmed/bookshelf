<script lang="ts">
  // Copyright © 2024 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.

  export let title: string = '';
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<div class="prose max-w-none">
  <h2 class="mb-1">{title}</h2>
  <hr class="mb-6" />
  <slot />
</div>

<style>
</style>
