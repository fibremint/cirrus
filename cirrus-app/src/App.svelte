<script lang="ts">
  // import logo from './assets/svelte.png'
  // import Counter from './lib/Counter.svelte'
  // import TestButton from './lib/TestButton.svelte';
  import { invoke } from '@tauri-apps/api';
  import { onMount } from 'svelte';

  import AudioList from './lib/AudioList.svelte';
  import AudioPlayer from './lib/AudioPlayer.svelte';

  // import type { AudioTag } from './types';

  // export let selectedAudioId: string = undefined;
  // export let audioTags: AudioTag[] = [];
  // export let selectedAudio: AudioTag = undefined;

  onMount(() => {
    console.log("invoke init audio player");
    invoke('plugin:cirrus|init_audio_player');
  })

</script>

<main>
  <AudioList />
  <!-- <AudioList audioTags={audioTags} bind:selectedAudio /> -->
  <!-- <AudioList bind:selectedAudio /> -->

  <AudioPlayer />
  <!-- <AudioPlayer audioTags={audioTags} selectedAudio={selectedAudio} /> -->
  <!-- <TestButton /> -->
  <!-- <img src={logo} alt="Svelte Logo" />
  <h1>Hello Typescript!</h1> -->

  <!-- <Counter />
  <TestButton /> -->

  <!-- <p>
    Visit <a href="https://svelte.dev">svelte.dev</a> to learn how to build Svelte
    apps.
  </p>

  <p>
    Check out <a href="https://github.com/sveltejs/kit#readme">SvelteKit</a> for
    the officially supported framework, also powered by Vite!
  </p> -->
  <!-- <link rel="stylesheet" href="node_modules/svelte-material-ui/bare.css" /> -->

</main>
<!-- 
<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  }

  main {
    text-align: center;
    padding: 1em;
    margin: 0 auto;
  }

  img {
    height: 16rem;
    width: 16rem;
  }

  h1 {
    color: #ff3e00;
    text-transform: uppercase;
    font-size: 4rem;
    font-weight: 100;
    line-height: 1.1;
    margin: 2rem auto;
    max-width: 14rem;
  }

  p {
    max-width: 14rem;
    margin: 1rem auto;
    line-height: 1.35;
  }

  @media (min-width: 480px) {
    h1 {
      max-width: none;
    }

    p {
      max-width: none;
    }
  }
</style> -->
