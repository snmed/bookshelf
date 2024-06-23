<script lang="ts">
  import Icon from '@/components/Icon.svelte';
  import { Icons } from '@/models/icons';
  import { _ } from 'svelte-i18n';
</script>

<svelte:head>
  <title>Bookshelf</title>
  <meta name="description" content="Bookshelf" />
</svelte:head>

<div class="flex flex-col gap-3">
  <div class="flex justify-center gap-2 flex-col sm:flex-row">
    <button class="btn w-full sm:w-32" title={$_('hints.create-db')}>
      <Icon class="h-5 w-5" name={Icons.PlusCircleLine}></Icon>
      {$_('labels.create')}
    </button>

    <button class="btn w-full sm:w-32" title={$_('hints.open-db')}>
      <Icon class="h-5 w-5" name={Icons.HardDiskSolid}></Icon>
      {$_('labels.open')}
    </button>
  </div>

  <div class="flex justify-center">
    <div class="flex flex-col w-full sm:w-10/12">
      <div class="flex justify-between gap-4 flex-col sm:flex-row mb-4">
        <div>
          <strong>Current Database:</strong>
          <div class="break-all">abscdf</div>
        </div>
        <button class="btn btn-accent" title={$_('hints.open-db')}>
          <Icon class="h-5 w-5" name={Icons.TimesCircleLine}></Icon>
          Close
        </button>
      </div>

      <div class="mb-2"><strong>Last items:</strong></div>
      <ul
        class="flex flex-col space-y-1 border rounded-md p-4 min-h-48 max-h-92"
      >
        <li
          class="flex flex-row gap-4 justify-between items-center bg-base-200/50 hover:bg-base-200 rounded p-2 cursor-pointer"
        >
          <span class="break-all">/home/sandro/documents/mybooks.db</span>
          <button
            class="btn btn-circle btn-error btn-outline btn-xs justify-self-end"
          >
            <Icon class="h-5 w-5" name={Icons.TimesLine}></Icon>
          </button>
        </li>
        <li
          class="flex flex-row gap-4 justify-between items-center bg-base-200/50 hover:bg-base-200 rounded p-2 cursor-pointer"
        >
          <span class="break-all">/tmp/mybooks.db</span>
          <button class="btn btn-circle btn-error btn-outline btn-xs">
            <Icon class="h-5 w-5" name={Icons.TimesLine}></Icon>
          </button>
        </li>
        <li
          class="flex flex-row gap-4 justify-between items-center bg-base-200/50 hover:bg-base-200 rounded p-2 cursor-pointer"
        >
          <span class="break-all"
            >/some/long/path/which/will/wrap/text/around/so/it/has/enough/space/mybooks.db</span
          >
          <button class="btn btn-circle btn-error btn-outline btn-xs">
            <Icon class="h-5 w-5" name={Icons.TimesLine}></Icon>
          </button>
        </li>
        <li
          class="flex flex-row gap-4 justify-between items-center bg-base-200/50 hover:bg-base-200 rounded p-2 cursor-pointer"
        >
          <span class="break-all">/var/mybooks.db</span>
          <button class="btn btn-circle btn-error btn-outline btn-xs">
            <Icon class="h-5 w-5" name={Icons.TimesLine}></Icon>
          </button>
        </li>
      </ul>
    </div>
  </div>
</div>

<!-- <section class="place-content-center">
  <div>
    <h1>
      <span class="welcome"> Hello</span>

      to your new<br />SvelteKit app
    </h1>

    <h2>
      try editing <strong>src/routes/+page.svelte</strong>
    </h2>
  </div>
</section> -->

<style lang="postcss">
  .welcome {
    background-color: blue;
  }
</style>
