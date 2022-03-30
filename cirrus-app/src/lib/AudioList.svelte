<!-- <div>
  <List
    class="demo-list"
    twoLine
    avatarList
    singleSelection
    bind:selectedIndex={selectionIndex}
  >
    {#each options as item}
      <Item
        on:SMUI:action={() => (selection = item.name)}
        disabled={item.disabled}
        selected={selection === item.name}
      >
        <Graphic
          style="background-image: url(https://place-hold.it/40x40?text={item.name
            .split(' ')
            .map((val) => val.substring(0, 1))
            .join('')}&fontsize=16);"
        />
        <Text>
          <PrimaryText>{item.name}</PrimaryText>
          <SecondaryText>{item.description}</SecondaryText>
        </Text>
        <Meta class="material-icons">info</Meta>
      </Item>
    {/each}
  </List>
</div>

<pre
  class="status">Selected: {selection}, value of selectedIndex: {selectionIndex}</pre>

<script lang="ts">
  import List, {
    Item,
    Graphic,
    Meta,
    Text,
    PrimaryText,
    SecondaryText,
  } from '@smui/list';

  let options = [
    {
      name: 'Bruce Willis',
      description: 'Actor',
      disabled: false,
    },
    {
      name: 'Austin Powers',
      description: 'Fictional Character',
      disabled: true,
    },
    {
      name: 'Thomas Edison',
      description: 'Inventor',
      disabled: false,
    },
    {
      name: 'Stephen Hawking',
      description: 'Scientist',
      disabled: false,
    },
  ];
  let selection = 'Stephen Hawking';
  // This value is updated when the component is initialized, based on the
  // selected Item's `selected` prop.
  let selectionIndex: number | undefined = undefined;
</script>

<style>
  /* * :global(.demo-list) {
    max-width: 600px;
  } */
</style> -->
<!-- 
<div>
  <List class="demo-list">
    <Item on:SMUI:action={() => (clicked = 'Cut')}><Text>Cut</Text></Item>
    <Item on:SMUI:action={() => (clicked = 'Copy')}><Text>Copy</Text></Item>
    <Item on:SMUI:action={() => (clicked = 'Paste')}><Text>Paste</Text></Item>
    <Item on:SMUI:action={() => (clicked = 'Delete')}><Text>Delete</Text></Item>
  </List>
</div>

<pre class="status">Clicked: {clicked}</pre>

<script lang="ts">
  import List, { Item, Separator, Text } from '@smui/list';

  let clicked = 'nothing yet';
</script>

<style>
  /* * :global(.demo-list) {
    max-width: 300px;
  } */
</style> -->

<script lang="ts">
  import { invoke } from '@tauri-apps/api';
  import List, { Item, Separator, Text } from '@smui/list';
  import {onMount} from 'svelte';
  import InfiniteScroll from 'svelte-infinite-scroll';

  export let selectedAudioId: string = undefined;
  
  let currentPage = 1;
  let itemsPerPage = 20;
  // let selectedId = "";

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

  // function getAudioTags() {
  //   invoke('plugin:cirrus|get_audio_tags', { itemsPerPage: 10, page: currentPage })
  //     .then((res) => {
  //       if (Array.isArray(res)) {
  //         audioTags = res;
  //       }
  //     });
  // }

  // function prevList() {
  //   if (currentPage > 1) {
  //     currentPage -= 1;
  //     getAudioTags();
  //   }
  // }

  // function nextList() {
  //   currentPage += 1;
  //   getAudioTags();
  // }

  // getAudioTags();
</script>

<div>
  <!-- <button on:click={prevList}>prev</button>
  <button on:click={nextList}>next</button> -->

  <!-- <List>
    {#each audioTags as item}
      <Item
        on:SMUI:action={() => selectedAudioId = item.id}>
        <Text>{item.artist} - {item.title}</Text>
      </Item>
    {/each}
  </List> -->
  <ul id="scrollContent">
    {#each audioTags as item}
      <li>{item.artist} - {item.title}</li>
    {/each}
    <InfiniteScroll
      hasMore={audioTagsFetch.length > 0} 
      threshold={itemsPerPage}
      on:loadMore={() => {currentPage++; fetchAudioTags()}} 
      horizontal={false}
      reverse={false}
      window={false}
      elementScroll={document.getElementById("scrollContent")} />
  </ul>
  <h5>
    All items loaded: {audioTagsFetch.length ? 'No' : 'Yes'}
  </h5>
  <!-- </InfiniteScroll> -->
  <!-- <pre class="status">selectedId: {selectedAudioId}</pre> -->
</div>

<style>
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
</style>
<!-- <script lang="ts">
  import { emit, listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api'

  let count: number = 0
  const increment = () => {
    count += 1
  }

  function invokeTest() {
    increment();
    invoke('plugin:cirrus|load_audio', { request: 'test-source2.aiff' });
  }
</script> -->
