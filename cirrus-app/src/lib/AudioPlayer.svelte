<script lang="ts">
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