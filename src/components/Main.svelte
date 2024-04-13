<script lang="ts">
  // Copyright © 2023 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.
  import SideMenu from '@/components/SideMenu.svelte';
  import Greet from '@/lib/Greet.svelte';
  import SplitLayout from '@/components/layout/SplitLayout.svelte';
</script>

<div class="bs-main-layout h-full w-full">
  <!-- <SideMenu />
    <div class="bs-main-container">
        <Greet />
    </div>    -->

  <SplitLayout isOpen={true} autoOpen={true}></SplitLayout>
</div>

<style>
  .bs-main-layout {
    display: flex;
    flex-direction: row;
  }
</style>
