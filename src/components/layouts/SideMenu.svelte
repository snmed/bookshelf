<script lang="ts">
  // Copyright © 2024 Sandro Dallo
  //
  // Use of this source code is governed by an BSD-style
  // license that can be found in the LICENSE file.
  import Icon from '@/components/Icon.svelte';
  import { Icons } from '@/models/icons';
  import { Themes, useThemeStore } from '@/api';
  import { onDestroy } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { useAppContext } from '@/contexts/app';
  import type { Unsubscriber } from 'svelte/motion';

  const {
    context: { menuOpen, toggleMenu },
  } = useAppContext();
  const themeStore = useThemeStore();

  const unsubscribers: Unsubscriber[] = [];

  let current: Themes = Themes.Dark;
  unsubscribers.push(
    themeStore.subscribe((t) => {
      current = t;
    }),
  );

  $: themeBtnTitle =
    current === Themes.Dark
      ? `Theme ${$_('labels.light-theme')}`
      : `Theme ${$_('labels.dark-theme')}`;

  onDestroy(() => {
    unsubscribers.forEach((u) => u());
  });
</script>

<nav class="bg-base-200">
  <!-- Top Menu -->
  <ul class="menu bg-base-200">
    <li>
      <a href="#top" title={$menuOpen ? '' : $_('labels.library')}>
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
        <span>{$_('labels.library')}</span>
      </a>
    </li>
  </ul>

  <!--Static Bottom Menu -->
  <ul class="menu bg-base-200">
    <li>
      <a
        role="button"
        tabindex="0"
        href="self"
        title={$menuOpen ? '' : themeBtnTitle}
        on:click|preventDefault={() =>
          themeStore.setTheme(
            current === Themes.Dark ? Themes.Light : Themes.Dark,
          )}
      >
        <Icon
          class="h-5 w-5"
          name={current === Themes.Dark ? Icons.SunLine : Icons.MoonLine}
        ></Icon>
        <span class="label"
          >{$_(
            current === Themes.Dark
              ? 'labels.light-theme'
              : 'labels.dark-theme',
          )}</span
        >
      </a>
    </li>
    <li>
      <a
        role="button"
        tabindex="0"
        href="self"
        title={$menuOpen ? '' : $_('labels.settings')}
      >
        <Icon class="h-5 w-5" name={Icons.CogSolid}></Icon>
        <span class="label">{$_('labels.settings')}</span>
      </a>
    </li>
    <li>
      <a
        role="button"
        tabindex="0"
        href="self"
        on:click|preventDefault={async () => await toggleMenu()}
      >
        <span class="h-5 w-5 collapse-btn" class:collapsed={!$menuOpen}>
          <Icon class="h-5 w-5" name={Icons.AngleDoubleLine}></Icon>
        </span>
      </a>
    </li>
  </ul>
</nav>

<style lang="postcss">
  nav {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;

    & ul > li {
      max-width: 100%;
    }

    & ul > li > a {
      overflow: hidden;
      white-space: nowrap;
      max-width: 100%;

      > *:nth-child(2) {
        margin-left: 1ch;
        padding: 0;
      }
    }

    .collapse-btn {
      transform: rotate(270deg);
      transition: transform 400ms ease;

      &.collapsed {
        transform: rotate(90deg);
      }
    }
  }
</style>
