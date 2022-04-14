<script lang="ts">
  import { invoke } from '@tauri-apps/api';
  
  import type { AudioTag } from '../types';
  import { audioTagsStore, selectedAudioTagStore } from '../state';

  export let audioTags: AudioTag[] = [];
  export let selectedAudioTag: AudioTag = undefined;

  audioTagsStore.subscribe(value => {
    audioTags = value;
  })

  selectedAudioTagStore.subscribe(value => {
    selectedAudioTag = value;
  })

  let isPlaying: boolean = false;

  function playAudio() {
    if (selectedAudioTag != undefined) {
      console.log('call play audio', selectedAudioTag.title);
      invoke('plugin:cirrus|load_audio', { audioTagId: selectedAudioTag.id });
    }
  }

  $: selectedAudioTag, playAudio();
</script>

<!-- <div> -->
  {#if selectedAudioTag}
    <p>selected audio id: {selectedAudioTag.id}</p>
    <p>selected item index: {audioTags.indexOf(selectedAudioTag)}</p>
    <!-- <p>selected item index: {audioTags)}</p> -->

  {:else}
    <p>select an audio</p>
  {/if}
<!-- </div> -->