<script>
  import { onMount, onDestroy } from 'svelte';

  import { 
    Navbar, 
    Page, 
    BlockTitle, 
    List, 
    ListItem, 
    Sheet, 
    Toolbar, 
    Link, 
    PageContent, 
    Block, 
    View, 
    Icon, 
    Range, 
    Button, 
    Segmented,
    Progressbar,
    BlockHeader,
  } from 'framework7-svelte';
  import { listen, emit } from '@tauri-apps/api/event';
  // import { audioStore } from '../js/store';

  import differenceBy from 'lodash/differenceBy';
  import * as command from '../js/command';

  let isLoading = false;

  let currentPage = 1;
  const itemsPerPage = 50;

  let audioTags = [];

  let isAudioPlay = false;
  let latestFetchDatetime = 0;

  let playbackContext = {
    audio: null,
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
  let loadedNextStream = false;

  const UPDATED_AUDIO_PLAYER_EVENT_NAME = "update-playback"

  onMount(async() => {
    const audioListMainNode = Array.from(document.getElementById('audio-list-main').children);
    const pageContent = audioListMainNode.find(e => e.classList.contains('page-content'))
    pageContent.style = "overflow: hidden;";

    const playerToolbarNode = Array.from(document.getElementById('player-toolbar').children);
    const toolbarInner = playerToolbarNode.find(e => e.classList.contains('toolbar-inner'))
    toolbarInner.style = "overflow: visible;";

    fetchAllAudio();

    const unlisten = await listen(UPDATED_AUDIO_PLAYER_EVENT_NAME, event => {      
      // const { messageType, message } = event.payload;
      const payload = event.payload;
      console.log("payload: ", payload);

      if (payload.messageType === "CurrentStream") {
        let currentAudio = audioTags.filter(item => item.id === payload.streamId)[0];
        console.log("currentAudio: ", currentAudio);

        playbackContext.audio = currentAudio;
        // currentStreamId = message.streamId;
        playbackContext.audioLength = Math.floor(payload.message.CurrentStream.length);
      }

      if (payload.messageType === "ResetState") {
        playbackContext.audio = null;
        playbackContext.audioLength = 0;
        playbackContext.position = 0;

        sliderPos = 0;
        
        updateAudioButton(false);

        nextAudio();
      }

      if (playbackContext.audio.id !== payload.streamId) {
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
      }
    });

    updatePlaybackPosEventUnlisten = unlisten;

    await command.setListenUpdatedEvents(true);

    await emit("stream-status");
  });

  async function nextAudio() {
    console.log('next audio');
    if (loadedNextStream) {
      console.log('already loaded next audio');
      return
    }

    loadedNextStream = true;

    if (audioTags.length <= loadedAudioItemIndex +1) {
      await fetchAudioTags();
    } 

    if (audioTags[loadedAudioItemIndex+1] === undefined) {
      loadedNextStream = false;
      return;
    }

    let nextAudioTag = audioTags[loadedAudioItemIndex+1];

    let loadedAudioMeta = await command.loadAudio(nextAudioTag.id);
    loadedAudioItemIndex += 1;

    if (!isAudioPlay) {
      await command.playAudio();
    }

    loadedNextStream = false;
  }

  onDestroy(async() => {
    console.log('destroy');
    updatePlaybackPosEventUnlisten();
    await command.stopAudio();
  });

  async function fetchAllAudio() {
    while (true) {
      console.log('fetch all audio');
      let fetchedItems = await fetchAudioTags();
      console.log('fetched num: ', fetchedItems)
      if (fetchedItems === 0 || fetchedItems === undefined) {
        break;
      }
    }
  }

  async function fetchAudioTags() {
    console.log("fetch audio tags")

    if (isLoading) {
      return;
    }

    isLoading = true;

    const response = await command.getAudioTags({ itemsPerPage, currentPage });
    if (!Array.isArray(response)) {
        console.log("failed to get audio tags");
        isLoading = false;

        return;
    }

    const uniqueItems = differenceBy(response, audioTags, "id");
    audioTags = [...audioTags, ...uniqueItems];

    if (uniqueItems.length === itemsPerPage) {
      currentPage++;
    }

    isLoading = false;

    return uniqueItems.length;
  }

  function fetchAudioTagsWithDelay() {
    const currentDateTime = Date.now();
    const isCalledWithinIdleTime = currentDateTime - latestFetchDatetime < 1000;
    latestFetchDatetime = currentDateTime;

    if (isCalledWithinIdleTime) return;

    fetchAudioTags();
  }

  function updateAudioButton(playStatus) {
    // ref: https://stackoverflow.com/questions/57874892/how-to-check-if-an-htmlcollection-contains-an-element-with-a-given-class-name-us
    const childrenNodeArr = Array.from(document.getElementById('play-pause-btn').children);
    const foundButtonInner = childrenNodeArr.find(e => e.classList.contains('icon'));

    if (foundButtonInner) {
      foundButtonInner.innerText = playStatus === true ? 'pause' : 'play_arrow';
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

  async function onAudioListItemClick({ itemIndex, audio }) {
    console.log(audio);
    if (playbackContext.audio !== null && 
      playbackContext.audio.id === audio.id) {

      console.log(`selected same audio`);

      return;
    }

    try {
      if (playbackContext.audioId !== null) {
        await command.stopAudio();
      }

      loadedAudioLength = await command.loadAudio(audio.id);
      // loadedAudioId = audioId;
      loadedAudioItemIndex = itemIndex;
      
      await command.playAudio();
    } catch(e) {
      console.log(`failed to play audio, ${e}`)
    }

    updateAudioButton(true);

    playbackContext.audio = audio;

    // console.log(`play audio: id: ${loadedAudioId}, content length: ${loadedAudioLength}`)
  }

  async function onPlayPauseButtonChange(event) {
    if (isAudioPlay) {
      await command.stopAudio();

      playbackContext.audio = null;
      playbackContext.audioLength = 0;
      playbackContext.position = 0;

      updateAudioButton(false);

      // resetSelectedAudioInfo();
    } else {
      await command.playAudio();
    }
  }

  async function handleForward() {
    await command.stopAudio();
    await nextAudio();
  }

</script>

<Page
  id="audio-list-main" >
  
  <Navbar title="All Music" backLink="Back" />

  <Progressbar infinite={isLoading} ></Progressbar>

  <Block
    style="
      width: 50%;
      margin-top: 16px;
      margin-bottom: 8px;" >

    <div 
      class="grid grid-gap"
      style="grid-template-columns: 20fr 40fr 40fr">
      <p>Sort by</p>
      <Button raised round>Artist</Button>
      <Button raised round>Music</Button>
    </div>
  </Block>
  
  <PageContent
    id="audio-list-container"
    infinite
    infiniteDistance={itemsPerPage}
    infinitePreloader={false}
    onInfinite={fetchAudioTagsWithDelay}
    style="padding-top: 0; margin-bottom: 48px" >

    <List 
      mediaList
      noHairlines
      style="margin: 0;" >

      {#each audioTags as audio, index}
        <ListItem
          title={` ${audio.title}`} 
          footer={audio.artist}
          link='#'
          on:click={onAudioListItemClick({ itemIndex: index, audio: audio })} />
      {/each}

      {#if isLoading}
        <ListItem
          class={`skeleton-text skeleton-effect-wave`}
          title="Skeleton Music Name Field" 
          footer="Skeleton Artist Name Field" />
      {/if}

    </List>
  
  </PageContent>

  <Toolbar bottom id="player-toolbar">
    <div style="width: 100%">
      <!-- recreate range component if 'audioLength' is changed -->
      {#key playbackContext.audioLength}
        <Range
          max={playbackContext.audioLength}
          step=1
          value={sliderPos}
          label={true}
          formatLabel={i => convertSecToMMSS(i)}
          onRangeChange={value => onAudioRangeChange(value)} 
          onRangeChanged={value => onAudioRangeChanged(value)} 
          style="
            position: relative;
            top: -6px;" />

      {/key}

      <div
        class="grid"
        style="
          grid-template-columns: 16fr 84fr;
          position: relative;
          top: -6px" >
        
        <div 
          class="grid"
          style="grid-template-columns: 10fr 10fr 10fr 70fr">
          <Button
            small={true}
            round
            iconMd="material:first_page"
            on:click={(e) => onPlayPauseButtonChange(e)} />

          <Button 
            small={true}
            round
            id="play-pause-btn" 
            iconMd={isAudioPlay === true ? "material:pause" : "material:play_arrow"}
            on:click={async(e) => {
              if (isAudioPlay) {
                await command.pauseAudio();
              } else {
                await command.playAudio();
              }

              updateAudioButton(!isAudioPlay);
            }} />

          <Button
            small={true}
            round
            iconMd="material:last_page"
            on:click={(e) => handleForward()} />
        </div>

        <div
          style="height: 40px; position: relative;" >

          {#if playbackContext.audio !== null}
            <BlockTitle
              style="margin: 0 0 0 10px"
              >{playbackContext.audio.title}</BlockTitle>
            <Block
              style="padding: 0 0 0 10px">
              <p>{playbackContext.audio.artist}</p>
            </Block>
          {/if}
        </div>

      </div>
      
    </div>

  </Toolbar>
</Page>
