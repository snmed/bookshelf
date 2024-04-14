<script lang="ts">
  // Copyright © 2023 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.
  import { Icons } from '@/models/icons';

  export let name: string | Icons = '';

$: icon = typeof name === 'string' ? name : Icons[name] ?? '';

</script>

<span class={`icon--clarity icon--clarity--${icon} ${$$props.class}`}></span>
