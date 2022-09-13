<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  import { event, invoke } from '@tauri-apps/api';
  import { emit, listen } from '@tauri-apps/api/event'
  import { appWindow, WebviewWindow } from '@tauri-apps/api/window';
  import Slider from '@smui/slider';

  import type { AudioTag, AudioPlayerContext, PlaybackPayload } from '../types';
  import { audioTagsStore, selectedAudioTagStore } from '../state';

  export let audioTags: AudioTag[] = [];
  export let selectedAudioTag: AudioTag = undefined;

  let prevAudioTag: AudioTag = undefined;
  let audioPlayerContext: AudioPlayerContext = {
    contentLength: 0.0,
    playbackPosition: 0.0,
    remainBuf: 0.0,
  };

  audioTagsStore.subscribe(value => {
    audioTags = value;
  })

  selectedAudioTagStore.subscribe(value => {
    selectedAudioTag = value;
  })

  let isPlaying: boolean = false;

  let updatePlaybackPosEventUnlisten = undefined;
  // const audioPlayerWebView = new WebviewWindow('audio-player');
  // const audioPlayerWebView = new appWindow;

  onMount(async () => {
    const unlisten = await listen<PlaybackPayload>('update-playback-pos', event => {
      const payload: PlaybackPayload = event.payload;
      // console.log("pos: %f, remain_buf: %f", payload.pos, payload.remainBuf);
      audioPlayerContext.playbackPosition = payload.pos;
      audioPlayerContext.remainBuf = payload.remainBuf;
    })

    updatePlaybackPosEventUnlisten = unlisten;

    await invoke('plugin:cirrus|send_playback_position');
  });

  onDestroy(() => {
    updatePlaybackPosEventUnlisten();
  });
  
  async function playAudio() {
    if (selectedAudioTag == undefined) {
      return;
    }

    if (selectedAudioTag != prevAudioTag) {
      console.log("sa: ", selectedAudioTag, ", pat: ", prevAudioTag);
      
      if (isPlaying) {
        console.log('call stop audio');
        await invoke('plugin:cirrus|stop_audio');
      }

      try {
        console.log('call play audio: ', selectedAudioTag.title);
        const contentLength: number = await invoke('plugin:cirrus|load_audio', { audioTagId: selectedAudioTag.id });
        await invoke('plugin:cirrus|start_audio');
        audioPlayerContext.contentLength = contentLength;

        isPlaying = true;
        prevAudioTag = selectedAudioTag;
      } catch (e) {
        console.log(e);

        isPlaying = false;
      }
    } else {
      console.log("selected same audio");
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

  {#if isPlaying}
    <Slider 
      style="flex-grow: 1;"
      bind:value={audioPlayerContext.playbackPosition} 
      max={audioPlayerContext.contentLength} 
    />
    <p>content length: {audioPlayerContext.contentLength}</p>
    <!-- <p>position: {audioPlayerContext.playbackPosition}</p>
    <p>buf: {audioPlayerContext.remainBuf}</p> -->
  {/if}
<!-- </div> -->