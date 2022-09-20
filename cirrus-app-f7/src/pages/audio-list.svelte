<script>
  import { onMount, onDestroy } from 'svelte';
  import { writable } from "svelte/store";

  import { Navbar, Page, BlockTitle, List, ListItem, Sheet, Toolbar, Link, PageContent, Block, View, Icon, ListItemCell, Range, Button, Segmented } from 'framework7-svelte';
  import { listen } from '@tauri-apps/api/event';
  import { audioStore } from '../js/store';

  import * as command from '../js/command';

  let allowInfinite = true;
  let showPreloader = true;

  // let isFetchItems = false;
  let currentPage = 1;
  const itemsPerPage = 20;
  let audioTags = [];
  let selectedAudioItemIdx = -1;
  let selectedAudioTagId = '';
  // let isHidePlayer = false;
  let isAudioPlay = false;
  // let playerIcon = 'play';
  let latestFetchDatetime = 0;

  let audioLength = 0;
  let currentPos = 0;
  // let remainBuf = 0;
  let audioPlayerStatus = 'Stop';
  let contentLength = 0;
  
  let updatePlaybackPosEventUnlisten = undefined;
  // let sliderProps = null;
  // let sliderProps = {
  //   max: 0
  // };

  // let audioRangeBarProps = {
  //   audioLength: 0,
  //   position: 0,
  //   isUserModifyPlaybackPos: false,
  // }

  let sliderPos = 0;
  let isUserModifyPlaybackPos = false;
  let userSetPos = 0;

  // const audioIsPlayStore = writable(false);

  // audioIsPlayStore.subscribe(value => {
  //   audioIsPlay = value;
  // });

  onMount(async() => {
    fetchAudioTags();

    const unlisten = await listen('update-playback-pos', event => {
      const payload = event.payload;
      console.log(`buf: ${payload.remainBuf}`)

      if (audioPlayerStatus === 'Play' && payload.status === 'Stop') {
        audioLength = 0;
        updateAudioButton(false);
      } else if (audioPlayerStatus === 'Stop' && payload.status === 'Play') {
        audioLength = contentLength;
        updateAudioButton(true);
      }

      currentPos = Math.floor(payload.pos);

      if (!isUserModifyPlaybackPos) {
        sliderPos = currentPos;
      }

      remainBuf = payload.remainBuf;
      audioPlayerStatus = payload.status;
    });

    updatePlaybackPosEventUnlisten = unlisten;

    await command.sendAudioPlayerStatus();
  });

  onDestroy(async() => {
    console.log('destroy');
    updatePlaybackPosEventUnlisten();
    await command.stopAudio();
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

    // const response = await invoke('plugin:cirrus|get_audio_tags', { itemsPerPage, page: currentPage });
    const response = await command.getAudioTags({ itemsPerPage, currentPage });
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
    if (currentPos !== sliderValue) {
      userSetPos = sliderValue;
      isUserModifyPlaybackPos = true;
    } else {
      isUserModifyPlaybackPos = false;
    }
  }

  async function onAudioRangeChanged(sliderValue) {
    if (currentPos !== sliderValue) {
      console.log(`user change done, modified value: ${sliderValue}`);
      await command.setPlaybackPosition(sliderValue);
    }

    isUserModifyPlaybackPos = false;
    userSetPos = 0;
  }

  async function onAudioListItemClick({ index, itemId }) {
    if (selectedAudioTagId === itemId) {
      console.log('clicked same audio');
      return;
    }

    try {
      if (isAudioPlay) {
        await command.stopAudio();
      }

      contentLength = await command.loadAudio(itemId);
      audioLength = contentLength;

      await command.playAudio();

      selectedAudioItemIdx = index;
      selectedAudioTagId = itemId;

      updateAudioButton(true);
    } catch(e) {
      console.log(`failed to play audio, ${e}`)
    }
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
        selected={index === selectedAudioItemIdx}
        title={` ${item.title}`} 
        footer={item.artist}
        link='#'
        on:click={onAudioListItemClick({ index, itemId: item.id })} />
    {/each}
  </List>

  <Toolbar bottom>
    <List simpleList style='width:100%'>
      <ListItem>
        <!-- <ListItemCell class="width-auto">
          <Button 
            iconF7='backward_fill' />
        </ListItemCell> -->

        <ListItemCell class="width-auto">
          <Button 
            id="play-pause-btn" 
            iconF7={isAudioPlay === true ? 'pause_fill' : 'play_fill'} 
            on:click={async(e) => {
              if (isAudioPlay) {
                await command.pauseAudio();
              } else {
                await command.playAudio();
              }

              updateAudioButton(!isAudioPlay);
            }} />
        </ListItemCell>

        <!-- <ListItemCell class="width-auto">
          <Button 
            iconF7='forward_fill' />
        </ListItemCell> -->

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
      </ListItem>
    </List>
  </Toolbar>
</Page>
