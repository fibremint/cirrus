<script lang="ts">
  import { invoke } from '@tauri-apps/api';
  
  import type { AudioTag } from '../types';
  import { audioTagsStore, selectedAudioTagStore } from '../state';

  export let audioTags: AudioTag[] = [];
  export let selectedAudioTag: AudioTag = undefined;

  let prevAudioTag: AudioTag = undefined;

  audioTagsStore.subscribe(value => {
    audioTags = value;
  })

  selectedAudioTagStore.subscribe(value => {
    selectedAudioTag = value;
  })

  let isPlaying: boolean = false;

  function playAudio() {
    // TODO: if audio is playing and selects another audio, stop previous audio and play selected audio
    if (selectedAudioTag == undefined) {
      return;
    }

    if (selectedAudioTag != prevAudioTag) {
      console.log("sa: ", selectedAudioTag, ", pat: ", prevAudioTag);
      
      if (isPlaying) {
        console.log('call stop audio');
        invoke('plugin:cirrus|stop_audio');
      }

      console.log('call play audio: ', selectedAudioTag.title);
      invoke('plugin:cirrus|load_audio', { audioTagId: selectedAudioTag.id });
      isPlaying = true;
      
      prevAudioTag = selectedAudioTag;
    } else {
      console.log("selected same audio");

    }
    // if (selectedAudioTag != undefined) {
    //   // console.log("sat: ", selectedAudioTag);
    //   // prevAudioTag = selectedAudioTag;

    //   console.log('call play audio: ', selectedAudioTag.title);
    //   invoke('plugin:cirrus|load_audio', { audioTagId: selectedAudioTag.id });
    // } 

    // if (selectedAudioTag != prevAudioTag) {
    //   console.log("selected another audio");
    // }
    // else {
    //   console.log("selected another audio")
    // }
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