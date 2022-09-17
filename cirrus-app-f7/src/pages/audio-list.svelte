<script>
  import { onMount, onDestroy } from 'svelte';
  import { writable } from "svelte/store";

  import { Navbar, Page, BlockTitle, List, ListItem, Sheet, Toolbar, Link, PageContent, Block, View, Icon, ListItemCell, Range, Button, Segmented } from 'framework7-svelte';
  import { invoke } from '@tauri-apps/api';

  import { audioStore } from '../js/store';
import { children, update_await_block_branch } from 'svelte/internal';
  

  let allowInfinite = true;
  let showPreloader = true;

  let isFetchItems = false;
  let currentPage = 1;
  const itemsPerPage = 20;
  let audioTags = [];
  let selectedItemIdx = -1;
  let isHidePlayer = false;
  let isAudioPlay = false;
  let playerIcon = 'play';
  let latestFetchDatetime = 0;

  // const audioIsPlayStore = writable(false);

  // audioIsPlayStore.subscribe(value => {
  //   audioIsPlay = value;
  // });

  onMount(() => {
    fetchAudioTags();
  });

  onDestroy(() => {
    console.log('destroy');
    invoke('plugin:cirrus|stop_audio');
  });

  async function fetchAudioTags() {
    const currentDateTime = Date.now();
    const isCalledWithinIdleTime = currentDateTime - latestFetchDatetime < 1000;
    latestFetchDatetime = currentDateTime;

    if (!allowInfinite || isCalledWithinIdleTime) return;
    console.log("fetch audio tags")

    // isFetchItems = true;
    allowInfinite = false;
    showPreloader = true;

    const response = await invoke('plugin:cirrus|get_audio_tags', { itemsPerPage, page: currentPage });
    
    if (!Array.isArray(response)) {
        console.log("failed to get audio tags");
        showPreloader = false;
        allowInfinite = true;
        return;
    }

    audioTags = [...audioTags, ...response];

    currentPage++;

    allowInfinite = true;
    showPreloader = false;
  }

  async function playAudio() {
    await invoke('plugin:cirrus|start_audio');
  }

  async function pauseAudio() {
    await invoke('plugin:cirrus|pause_audio');
  }

  function updateAudioButton(playStatus) {
    // ref: https://stackoverflow.com/questions/57874892/how-to-check-if-an-htmlcollection-contains-an-element-with-a-given-class-name-us
    const childrenNodeArr = Array.from(document.getElementById('play-pause-btn').children);
    const foundButtonInner = childrenNodeArr.find(e => e.classList.contains('icon'));

    if (foundButtonInner) {
      foundButtonInner.innerText = playStatus === true ? 'pause_fill' : 'play_fill';
    }

    isAudioPlay = playStatus;
  }

  // $: playerIconProp = isPlay ? 'pause_fill' : 'play_fill';
</script>
<!-- <Page> -->
<Page  
  infinite
  infiniteDistance={itemsPerPage}
  infinitePreloader={showPreloader}
  onInfinite={fetchAudioTags}>

  <Navbar title="Audio list" backLink="Back" />

  <List mediaList noHairlines>
    {#each audioTags as item, index (index)}
      <ListItem 
        title={` ${item.title}`} 
        footer={item.artist}
        link='#'
        on:click={async(e) => {
          // audioStore.dispatch('setSelectedAudioTag', {
          //   selectedAudioTag: item
          // });

          // audioStore.dispatch('setIsHidePlayer', false);
          // audioStore.dispatch('setIsHidePlayer', {
          //   isHidePlayer: false
          // });
          // isHidePlayer = false;
        
          // selectedItemIdx = index;

          // console.log(selectedItemIdx);

          const contentLength = await invoke('plugin:cirrus|load_audio', { audioTagId: item.id });
          console.log(contentLength);

          await invoke('plugin:cirrus|start_audio');

          updateAudioButton(!isAudioPlay);
        }}
      />
    {/each}
  </List>

  <Toolbar bottom>
    <List simpleList style='width:100%'>
      <ListItem>
        <ListItemCell class="width-auto flex-shrink-0">
          <!-- <a class="button" href="#" f7>
            <i clsas="icon f7-icons">{audioIsPlay === true ? 'pause_fill' : 'play_fill'}</i>
          </a> -->
          <Button 
            id="play-pause-btn" 
            iconF7={isAudioPlay === true ? 'pause_fill' : 'play_fill'} 
            on:click={(e) => {
              if (isAudioPlay) {
                // pause audio if audio is playing
                console.log('click pause btn');
                pauseAudio();
              } else {
                // play audio if audio is paused
                console.log('click play btn');
                playAudio();
              }

              updateAudioButton(!isAudioPlay);
            }} >
          </Button>
          <!-- <Icon ios="f7:speaker_fill" aurora="f7:speaker_fill" md="material:volume_mute"></Icon> -->
        </ListItemCell>
        <ListItemCell class="flex-shrink-3">
          <Range
            min={0}
            max={100}
            step={1}
            value={10}
          ></Range>
        </ListItemCell>
        <ListItemCell class="width-auto flex-shrink-0">
          <Icon ios="f7:speaker_3_fill" aurora="f7:speaker_3_fill" md="material:volume_up"></Icon>
        </ListItemCell>
      </ListItem>
    </List>
  </Toolbar>
</Page>
