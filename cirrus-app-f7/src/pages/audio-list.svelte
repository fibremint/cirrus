<script>
  import { onMount, onDestroy } from 'svelte';
  import { writable } from "svelte/store";

  import { Navbar, Page, BlockTitle, List, ListItem, Sheet, Toolbar, Link, PageContent, Block, View, Icon, ListItemCell, Range, Button, Segmented } from 'framework7-svelte';
  import { invoke } from '@tauri-apps/api';
  import { listen } from '@tauri-apps/api/event';
  import { audioStore } from '../js/store';

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

  let audioLength = 0;
  let currentPos = 0;
  let remainBuf = 0;
  let audioPlayerStatus = 'Stop';
  
  let updatePlaybackPosEventUnlisten = undefined;
  // let sliderProps = null;
  let sliderProps = {
    max: 0
  };

  let audioRangeBarProps = {
    audioLength: 0,
    position: 0,
    isUserModifyPlaybackPos: false,
  }

  let sliderPos = 0;
  let isUserModifyPlaybackPos = false;
  let userSetPos = 0;

  // let isAudioRangeBar

  // const audioIsPlayStore = writable(false);

  // audioIsPlayStore.subscribe(value => {
  //   audioIsPlay = value;
  // });

  onMount(async() => {
    fetchAudioTags();

    const unlisten = await listen('update-playback-pos', event => {
      const payload = event.payload;
      // console.log(payload);

      if (audioPlayerStatus === 'Play' && payload.status === 'Stop') {
        audioLength = 0;
        updateAudioButton(false);
      }

      currentPos = Math.floor(payload.pos);

      if (!isUserModifyPlaybackPos) {
        sliderPos = currentPos;
      }

      remainBuf = payload.remainBuf;
      audioPlayerStatus = payload.status;
    });

    updatePlaybackPosEventUnlisten = unlisten;

    await invoke('plugin:cirrus|send_audio_player_status');
  });

  onDestroy(() => {
    console.log('destroy');
    updatePlaybackPosEventUnlisten();
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

  async function setPlaybackPosition(positionSec) {
    console.log(positionSec);
    await invoke('plugin:cirrus|set_playback_position', {playbackPos: positionSec});
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

  function convertSecToMMSS(seconds) {
    // ref: https://stackoverflow.com/a/1322771
    return new Date(seconds * 1000).toISOString().substring(14, 19)
  }

  // $: playerIconProp = isPlay ? 'pause_fill' : 'play_fill';

  function onAudioRangeChange(sliderValue) {
    // console.log(`curretPos: ${currentPos}, sliderValue: ${sliderValue}`)
    if (currentPos !== sliderValue) {
      userSetPos = sliderValue;
      isUserModifyPlaybackPos = true;
    } else {
      isUserModifyPlaybackPos = false;
    }
  }

  function onAudioRangeChanged(sliderValue) {
    if (currentPos !== sliderValue) {
      console.log(`user change done, modified value: ${sliderValue}`);
      setPlaybackPosition(sliderValue);
    }

    isUserModifyPlaybackPos = false;
    userSetPos = 0;
  }

</script>
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
          const contentLength = await invoke('plugin:cirrus|load_audio', { audioTagId: item.id });
          console.log(contentLength);
          audioLength = contentLength;

          await invoke('plugin:cirrus|start_audio');

          // isAudioPlay = !isAudioPlay;
          updateAudioButton(!isAudioPlay);
        }}
      />
    {/each}
  </List>

  <Toolbar bottom>
    <List simpleList style='width:100%'>
      <ListItem>
        <ListItemCell class="width-auto flex-shrink-0">
          <Button 
            id="play-pause-btn" 
            iconF7={isAudioPlay === true ? 'pause_fill' : 'play_fill'} 
            on:click={(e) => {
              if (isAudioPlay) {
                pauseAudio();
              } else {
                playAudio();
              }

              updateAudioButton(!isAudioPlay);
            }} />
        </ListItemCell>
        <ListItemCell class="width-auto flex-shrink-0">
          <!-- <p>{convertSecToMMSS(sliderPos)}</p> -->
          {#if isUserModifyPlaybackPos}
            <p>{convertSecToMMSS(userSetPos)}</p>
          {:else }
            <p>{convertSecToMMSS(currentPos)}</p>
          {/if}
        </ListItemCell>
        <ListItemCell class="flex-shrink-3">
          <!-- recreate range component if 'audioLength' is changed -->
          {#key audioLength}
            <Range
              max={audioLength}
              step=1
              value={sliderPos}
              onRangeChange={value => onAudioRangeChange(value)} 
              onRangeChanged={value => onAudioRangeChanged(value)} />
          {/key}
        </ListItemCell>
        <ListItemCell class="width-auto flex-shrink-0">
          <p>{convertSecToMMSS(audioLength)}</p>
        </ListItemCell>
        <ListItemCell class="width-auto flex-shrink-0">
          <Icon ios="f7:speaker_3_fill" aurora="f7:speaker_3_fill" md="material:volume_up"></Icon>
        </ListItemCell>
      </ListItem>
    </List>
  </Toolbar>
</Page>
