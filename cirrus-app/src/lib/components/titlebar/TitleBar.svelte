<script lang="ts">
  import { appWindow, getCurrent } from '@tauri-apps/api/window';
  import { listen } from '@tauri-apps/api/event'
  import * as os from '@tauri-apps/api/os'
  
  import Icon from '@iconify/svelte';
  import chromeMinimize from '@iconify/icons-codicon/chrome-minimize';
  import chromeMaximize from '@iconify/icons-codicon/chrome-maximize';
  import chromeRestore from '@iconify/icons-codicon/chrome-restore';
  import chromeClose from '@iconify/icons-codicon/chrome-close';

  // import Button from './Button.svelte.1';
  // import { heroIcons as hi } from '../../icons';
	import { onMount } from 'svelte';

  let platform: string = '';

  onMount(async() => {
    platform = await os.platform();
  });

  let isWindowMaximized: Boolean = false;

  onMount(async() => {
    const window = getCurrent();
    isWindowMaximized = await window.isMaximized();
    listen('tauri://resize', async() => {
      isWindowMaximized = await window.isMaximized();
    });
  })

  function minimize() {
    getCurrent().minimize();
  }

  async function toggleMaximize() {
    const window = getCurrent();
    (await window.isMaximized()) ? window.unmaximize() : window.maximize();
  }

  function close() {
    getCurrent().close();
  }

</script>

{#if platform === 'win32'}
  <div
    class="fixed w-screen select-none h-8 pl-2 flex justify-between items-center text-black dark:text-gray-300"
    data-tauri-drag-region
  >
    <!-- <span class="items-start: pl-10">Cirrus</span> -->

    <!-- <div class="navbar bg-base-100">
      <div class="flex-1">
        <a class="btn btn-ghost normal-case" href="/">Cirrus</a>
      </div>
      <div class="flex-none gap-2">
        <div class="form-control">
          <input type="text" placeholder="Search" class="input input-bordered" />
        </div>
      </div>
    </div> -->

    <span class="items-start: pl-10">
      <a class="btn btn-ghost normal-case text-lg" href="/">Cirrus</a>
    </span>

      <!-- <span class="navbar bg-base-100">
        <div class="flex-1">
          <a class="btn btn-ghost normal-case text-lg" href="/">Cirrus</a>
        </div>
        <div class="flex-none gap-2">
          <div class="form-control">
            <input type="text" placeholder="Search" class="input input-bordered" />
          </div>
        </div>
      </span> -->
    
    <!-- ref: https://tailwindcss.com/blog/tailwindcss-v3-1#arbitrary-values-but-for-variants -->
    <span class="
      h-full
      [&>*]:h-full [&>*]:w-12 [&>*]:inline-flex
      [&>*]:items-center [&>*]:justify-center
    ">
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span
        title="Minimize"
        class="
          hover:bg-button-hover-light
          active:bg-button-active-light
          dark:hover:bg-button-hover-dark
          dark:active:bg-button-active-dark
          transition-all
        "
        on:click={minimize}
      >
        <div><Icon icon={chromeMinimize} /></div>
      </span>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span
        title={isWindowMaximized ? 'Restore' : 'Maximize'}
        class="
          hover:bg-button-hover-light
          active:bg-button-active-light
          dark:hover:bg-button-hover-dark
          dark:active:bg-button-active-dark
          transition-all
        "
        on:click={toggleMaximize}
      >
        {#if isWindowMaximized}
          <Icon icon={chromeRestore} />
        {:else}
          <Icon icon={chromeMaximize} />
        {/if}
      </span>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span
        title="Close"
        class="
          hover:bg-button-red-light
          hover:text-gray-300
          active:bg-button-red-light
          dark:hover:bg-button-red-dark
          dark:active:bg-button-red-dark
          transition-all
        "
        on:click={close}
      >
        <Icon icon={chromeClose} />
      </span>
      
    </span>  
    
  </div>
{/if}

<!-- 
<div>
  <div data-tauri-drag-region class="h-[30px] select-none flex justify-end fixed top-0 left-0 right-0">
    {#if platform === 'win32'}
      <Button 
        id="titlebar-minimize"
        svgData={hi.arrowDown} 
        onEvent={() => appWindow.minimize()} />
      
      <Button
        id="titlebar-maximize"
        svgData={hi.arrowTopRightOn}
        onEvent={() => appWindow.maximize()} />
      
      <Button
        id="titlebar-close"
        svgData={hi.xMark}
        onEvent={() => appWindow.close()} />
    {/if}
  </div>
</div>
 -->
