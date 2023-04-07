<Page name="home">
  <!-- Top Navbar -->
  <Navbar
    title="Cirrus"
    large={false}
    sliding={false}
    noHairline={true} >

    <!-- <NavTitle sliding>Cirrus</NavTitle> -->
    <NavRight>
      <!-- <Link iconIos="f7:setting" iconAurora="f7:menu" iconMd="material:settings" panelOpen="left" /> -->
      <Link iconMd="material:settings" panelOpen="left" />
    </NavRight>
    <!-- <NavRight>
      <Link iconIos="f7:menu" iconAurora="f7:menu" iconMd="material:menu" panelOpen="right" />
    </NavRight> -->
    <NavTitleLarge>Cirrus</NavTitleLarge>
  </Navbar>
  <!-- Toolbar -->
  <!-- <Toolbar bottom>
    <Link>Left Link</Link>
    <Link>Right Link</Link>
  </Toolbar> -->
  <!-- Page content -->
  <!-- <Block strong>
    <p>Here is your blank Framework7 app. Let's see what we have here.</p>
  </Block> -->

  <BlockTitle>Music</BlockTitle>
  <List>
    <ListItem link='/audio-list/' title="All"/>
  </List>

  <!-- <BlockTitle>Settings</BlockTitle>
  <List>
    <ListItem link='#' title="Setting"/>
  </List> -->

</Page>

<script>
  import {
    Page,
    Navbar,
    // NavLeft,
    NavTitle,
    NavTitleLarge,
    Link,
    BlockTitle,
    List,
    ListItem,
    NavRight,
  } from 'framework7-svelte';

</script>