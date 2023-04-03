<script>
  import { onMount, onDestroy } from 'svelte';

  import { Navbar, Page, BlockTitle, List, ListItem, Sheet, Toolbar, Link, PageContent, Block, View, Icon, ListItemCell, Range, Button, Segmented } from 'framework7-svelte';
  import { listen, emit } from '@tauri-apps/api/event';
  // import { audioStore } from '../js/store';

  import differenceBy from 'lodash/differenceBy';

  import * as command from '../js/command';
  import { filter } from 'dom7';
    import { message } from '@tauri-apps/api/dialog';

  let allowInfinite = true;
  let showPreloader = true;

  let currentPage = 1;
  const itemsPerPage = 50;

  let audioTags = [];

  let isAudioPlay = false;
  let latestFetchDatetime = 0;

  let playbackContext = {
    audioId: '',
    audioLength: 0,
    selectedAudioItemIndex: -1,
    position: 0,
  } 
  
  let updatePlaybackPosEventUnlisten = undefined;

  let sliderPos = 0;
  let isUserModifyPlaybackPos = false;
  let userSetPos = 0;

  let loadedAudioLength = 0;
  let loadedAudioItemIndex = -1;
  let loadedAudioId = '';
  let loadedNextStream = false;

  let currentStreamId = '';

  const UPDATED_AUDIO_PLAYER_EVENT_NAME = "update-playback"

  onMount(async() => {
    fetchAudioTags();

    const unlisten = await listen(UPDATED_AUDIO_PLAYER_EVENT_NAME, event => {      
      // const { messageType, message } = event.payload;
      const payload = event.payload;
      console.log("payload: ", payload);

      if (payload.messageType === "CurrentStream") {
        if (currentStreamId !== payload.streamId) {
          currentStreamId = payload.streamId;
          loadedNextStream = false;
          playbackContext.audioLength = 0;
          playbackContext.position = 0;
          sliderPos = 0;
        }
        // currentStreamId = message.streamId;
        playbackContext.audioLength = Math.floor(payload.message.CurrentStream.length);
      }

      if (payload.messageType === "ResetState") {
        currentStreamId = '';
        playbackContext.audioId = '';
        playbackContext.audioLength = 0;
        playbackContext.position = 0;
        sliderPos = 0;

        updateAudioButton(false);
      }

      if (currentStreamId !== payload.streamId) {
        return;
      }

      if (payload.messageType === "StreamStatus") {
        // if (payload.message.StreamStatus === "ReachEnd") {
        //   console.log("Reach end");
        // }

        let isAudioPlay = payload.message.StreamStatus === "Play" ? true : false;
        updateAudioButton(isAudioPlay);

      } else if (payload.messageType === "PositionSec") {
        playbackContext.position = payload.message.PositionSec;
      
        if (!isUserModifyPlaybackPos) {
          sliderPos = playbackContext.position;
        }

        // Load next audio stream
        if (playbackContext.audioLength - playbackContext.position > 5) {
          return;
        }

        nextAudio();
      }
    });

    updatePlaybackPosEventUnlisten = unlisten;

    await command.setListenUpdatedEvents(true);

    await emit("stream-status");
  });

  async function nextAudio() {
    if (loadedNextStream) {
      return
    }

    if (audioTags.length <= loadedAudioItemIndex +1) {
      await fetchAudioTags();
    } 

    if (audioTags[loadedAudioItemIndex+1] === undefined) {
      return;
    }

    let nextAudioTag = audioTags[loadedAudioItemIndex+1];

    let loadedAudioMeta = await command.loadAudio(nextAudioTag.id);
    loadedNextStream = true;
    loadedAudioItemIndex += 1;

    if (!isAudioPlay) {
      await command.playAudio();
    }
  }

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

    allowInfinite = false;
    showPreloader = true;

    const response = await command.getAudioTags({ itemsPerPage, currentPage });
    if (!Array.isArray(response)) {
        console.log("failed to get audio tags");
        showPreloader = false;
        allowInfinite = true;
        return;
    }

    const uniqueItems = differenceBy(response, audioTags, "id");
    audioTags = [...audioTags, ...uniqueItems];

    if (uniqueItems.length === itemsPerPage) {
      currentPage++;
    }

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
    if (playbackContext.position !== sliderValue) {
      userSetPos = sliderValue;
      isUserModifyPlaybackPos = true;
    } else {
      isUserModifyPlaybackPos = false;
    }
  }

  async function onAudioRangeChanged(sliderValue) {
    if (playbackContext.position !== sliderValue) {
      console.log(`move playback position: playback time: ${convertSecToMMSS(sliderValue)}, slide value: ${sliderValue}`);
      await command.setPlaybackPosition(sliderValue);
    }

    isUserModifyPlaybackPos = false;
    userSetPos = 0;
  }

  async function onAudioListItemClick({ itemIndex, audioId }) {
    if (playbackContext.audioId === audioId) {
      console.log(`selected same audio`);

      return;
    }

    try {
      if (playbackContext.audioId !== null) {
        await command.stopAudio();
      }

      loadedAudioLength = await command.loadAudio(audioId);
      loadedAudioId = audioId;
      loadedAudioItemIndex = itemIndex;
      
      await command.playAudio();
    } catch(e) {
      console.log(`failed to play audio, ${e}`)
    }

    updateAudioButton(true);

    console.log(`play audio: id: ${loadedAudioId}, content length: ${loadedAudioLength}`)
  }

  async function onPlayPauseButtonChange(event) {
    if (isAudioPlay) {
      await command.stopAudio();

      currentStreamId = '';
      playbackContext.audioLength = 0;
      playbackContext.position = 0;

      updateAudioButton(false);

      // resetSelectedAudioInfo();
    } else {
      await command.playAudio();
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
        selected={index === playbackContext.selectedAudioItemIndex}
        title={` ${item.title}`} 
        footer={item.artist}
        link='#'
        on:click={onAudioListItemClick({ itemIndex: index, audioId: item.id })} />
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
            <p>{convertSecToMMSS(playbackContext.position)}</p>
          {/if}
        </ListItemCell>

        {#key playbackContext.audioLength}
          <ListItemCell class="flex-shrink-3">
            <!-- recreate range component if 'audioLength' is changed -->
              <Range
                max={playbackContext.audioLength}
                step=1
                value={sliderPos}
                onRangeChange={value => onAudioRangeChange(value)} 
                onRangeChanged={value => onAudioRangeChanged(value)} />
          </ListItemCell>
        
          <ListItemCell class="width-auto flex-shrink-0">
            <p>{convertSecToMMSS(playbackContext.audioLength)}</p>
          </ListItemCell>
        {/key}

        <ListItemCell class="width-auto flex-shrink-0">
          <Button 
            id="play-pause-btn" 
            iconF7='xmark'
            on:click={(e) => onPlayPauseButtonChange(e)}/>        
        </ListItemCell>
      </ListItem>
    </List>
  </Toolbar>
</Page>
