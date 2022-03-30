<script lang="ts">
  import { invoke } from '@tauri-apps/api';
  import List, { Item, Text, PrimaryText, SecondaryText } from '@smui/list';
  import {onMount} from 'svelte';
  import InfiniteScroll from 'svelte-infinite-scroll';

  export let selectedAudioId: string = undefined;
  
  let selectedItemIdx: number | undefined = undefined;
  let currentPage = 1;
  let itemsPerPage = 20;

  type AudioTag = {
    id: string,
    artist: string,
    genre: string,
    title: string,
  };

  let audioTags: AudioTag[] = [];
  let audioTagsFetch: AudioTag[] = [];

  $: audioTags = [
    ...audioTags,
    ...audioTagsFetch,
  ];

  onMount(() => {
    fetchAudioTags();
  })

  async function fetchAudioTags() {
    const response = await invoke('plugin:cirrus|get_audio_tags', { itemsPerPage, page: currentPage });
    if (Array.isArray(response)) {
      console.log("fetch new data: ", response);
      audioTagsFetch = response;
    }
  }
</script>

<div class="audio-tag-list">
  <List
    id="scrollContent"
    class="audio-tag-list-content"
    twoLine
    singleSelection
    bind:selectedIndex={selectedItemIdx} 
  >
    {#each audioTags as item}
      <Item
        on:SMUI:action={() => {
          selectedAudioId = item.id; 
          selectedItemIdx = audioTags.indexOf(item)}}
        selected={selectedAudioId === item.id} 
      >
        <Text>
          <PrimaryText>{item.title}</PrimaryText>
          <SecondaryText>{item.artist}</SecondaryText>
        </Text>
      </Item>
    {/each}
    <InfiniteScroll
      hasMore={audioTagsFetch.length > 0} 
      threshold={itemsPerPage}
      on:loadMore={() => {currentPage++; fetchAudioTags()}} 
      horizontal={false}
      reverse={false}
      window={false}
      elementScroll={document.getElementById("scrollContent")} />
  </List>
  <!-- <h5>
    All items loaded: {audioTagsFetch.length ? 'No' : 'Yes'}
  </h5> -->
  <!-- <pre class="status">selectedId: {selectedAudioId}</pre> -->
</div>

<style>
  .audio-tag-list :global(.audio-tag-list-content) {
    /* align-items: left; */
    max-height: 100%;
    overflow: auto;
  }
  .audio-tag-list {
    display: flex;
    /* height: 100%; */
    /* align-items: left; */
    max-height: 600px;
  }
</style>

<!-- <style>
  main {
    display: flex;
    width: 100%;
    height: 100%;
    align-items: center;
    justify-content: center;
    flex-direction: column;
  }

  ul {
    box-shadow: 0px 1px 3px 0px rgba(0, 0, 0, 0.2),
      0px 1px 1px 0px rgba(0, 0, 0, 0.14), 0px 2px 1px -1px rgba(0, 0, 0, 0.12);
    display: flex;
    flex-direction: column;
    border-radius: 2px;
    width: 100%;
    max-width: 400px;
    max-height: 400px;
		background-color: white;
    overflow-x: scroll;
    list-style: none;
    padding: 0;
  }

  li {
    padding: 15px;
    box-sizing: border-box;
    transition: 0.2s all;
    font-size: 14px;
  }

  li:hover {
    background-color: #eeeeee;
  }
</style> -->