<script>
  import { onMount, onDestroy } from 'svelte';
  import { writable } from "svelte/store";

  import { Navbar, Page, BlockTitle, List, ListItem, Sheet, Toolbar, Link, PageContent, Block, View, Icon, ListItemCell, Range, Button, Segmented } from 'framework7-svelte';
  import { invoke } from '@tauri-apps/api';

  import { audioStore } from '../js/store';
import { update_await_block_branch } from 'svelte/internal';
  

  let allowInfinite = true;
  let showPreloader = true;

  let isFetchItems = false;
  let currentPage = 1;
  const itemsPerPage = 20;
  let audioTags = [];
  let selectedItemIdx = -1;
  let isHidePlayer = false;
  let audioIsPlay = false;
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

  async function pauseAudio() {
    await invoke('plugin:cirrus|pause_audio');
  }

  // async function playAudio(audioId) {
  //   const contentLength = await invoke('plugin:cirrus|load_audio', { audioTagId: audioId });
  //   await invoke('plugin:cirrus|start_audio');   
  // }

  // $: playerIconProp = isPlay ? 'pause_fill' : 'play_fill';
</script>
<!-- <Page> -->
<Page  
  infinite
  infiniteDistance={itemsPerPage}
  infinitePreloader={showPreloader}
  onInfinite={fetchAudioTags}>

  <Navbar title="Audio list" backLink="Back" />

  <!-- <Toolbar bottom style='heigth: auto'> -->
    <!-- <List> -->
      <!-- <ListItem>
        <ListItemCell class='width:auto flex-shrink-0'>
          {#if isPlay}
            <Icon f7="pause" />
          {:else}
            <Icon f7="play" />
          {/if}
        </ListItemCell>
        <ListItemCell class='flex-shrink-3'>
          <Range min={0} max={100}></Range>
        </ListItemCell>
        <ListItemCell class='widht:auto flex-shrink-0'>
          <Icon f7='close' />
        </ListItemCell>
      </ListItem> -->

      <!-- <ListItem style='width:100%'>
        <ListItemCell class="width-auto flex-shrink-0">
          <Icon ios="f7:speaker_fill" aurora="f7:speaker_fill" md="material:volume_mute"></Icon>
        </ListItemCell>
        <ListItemCell class="flex-shrink-3" style="width: 600px">
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

    </List> -->
    <!-- <Icon f7=${playerIcon} />
     -->
    <!-- ${selectedItemIdx} -->
    <!-- {isPlay ? "playing" : "paused"} -->
  <!-- </Toolbar> -->

  <!-- <Sheet> -->

    <!-- <List simpleList>
      <ListItem> 
        <ListItemCell class='width:auto flex-shrink-0'>
          {#if isPlay}
            <Icon f7="pause" />
          {:else}
            <Icon f7="play" />
          {/if}
        </ListItemCell>
        <ListItemCell class='flex-shrink-3'>
          <Range min={0} max={100}></Range>
        </ListItemCell>
        <ListItemCell class='widht:auto flex-shrink-0'>
          <Icon f7='close' />
        </ListItemCell>
      </ListItem>

    </List> -->
  <!-- </Sheet> -->

  <List mediaList noHairlines>
    {#each audioTags as item, index (index)}
      <ListItem 
        title={` ${item.title}`} 
        footer={item.artist}
        link='#'
        on:click={async(e) => {
          audioStore.dispatch('setSelectedAudioTag', {
            selectedAudioTag: item
          });

          // audioStore.dispatch('setIsHidePlayer', false);
          audioStore.dispatch('setIsHidePlayer', {
            isHidePlayer: false
          });
          isHidePlayer = false;
        
          selectedItemIdx = index;

          console.log(selectedItemIdx);

          audioIsPlay = true;
          playerIcon = 'pause';

          const playPauseBtn = document.getElementById('play-pause-btn');
          const innerContent = playPauseBtn.children[0];
          innerContent.innerText = audioIsPlay === true ? 'pause_fill' : 'play_fill';
          // audioIsPlayStore.update(value => value = true);
          
          // audioPropStore.store('pause_fill')

          // playAudio(item.id);
          const contentLength = await invoke('plugin:cirrus|load_audio', { audioTagId: item.id });
          await invoke('plugin:cirrus|start_audio');       
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
            iconF7={audioIsPlay === true ? 'pause_fill' : 'play_fill'} 
            on:click={(e) => {
              console.log('click play pause btn');
              pauseAudio();
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
