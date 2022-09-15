<script>
  import { onMount } from 'svelte';
  import { Navbar, Page, BlockTitle, List, ListItem, Sheet, Toolbar, Link, PageContent, Block, View, Icon, ListItemCell, Range } from 'framework7-svelte';
  import { invoke } from '@tauri-apps/api';

  import { audioStore } from '../js/store';

  let allowInfinite = true;
  let showPreloader = true;

  let isFetchItems = false;
  let currentPage = 1;
  const itemsPerPage = 20;
  let audioTags = [];
  let selectedItemIdx = -1;
  let isHidePlayer = false;
  let isPlay = false;
  let playerIcon = 'play';

  onMount(() => {
    fetchAudioTags();
  });

  async function fetchAudioTags() {
      if (!allowInfinite) return;
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

</script>
<!-- <Page> -->
<Page  
  infinite
  infiniteDistance={itemsPerPage}
  infinitePreloader={showPreloader}
  onInfinite={fetchAudioTags}>

  <Navbar title="Audio list" backLink="Back" />

  <Toolbar bottom style='heigth: auto'>
    <List>
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

      <ListItem style='width:100%'>
        <ListItemCell class="width-auto flex-shrink-0">
          <Icon ios="f7:speaker_fill" aurora="f7:speaker_fill" md="material:volume_mute"></Icon>
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
    <!-- <Icon f7=${playerIcon} />
     -->
    <!-- ${selectedItemIdx} -->
    <!-- {isPlay ? "playing" : "paused"} -->
  </Toolbar>

  <List mediaList>
    {#each audioTags as item, index (index)}
      <ListItem 
        title={` ${item.title}`} 
        footer={item.artist}
        on:click={e => {
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

          isPlay = true;
          playerIcon = 'pause';
        }}
      />
    {/each}
  </List>

</Page>
