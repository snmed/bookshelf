<script lang="ts">
  import type { MenuItem } from "@/models/menu";

// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

export let menus: MenuItem = [];


</script>

<aside class="bs-sidebar">


  <ul class="menu bg-base-200 w-56">
    <li>
      <a href="#top">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" /></svg>
      Item 2
      </a>
    </li>
  </ul>
</aside>

<style>
  .bs-sidebar {
    display: flex;
    height: 100%;
    width: 360px;
    background-color: blueviolet;
  }

</style>