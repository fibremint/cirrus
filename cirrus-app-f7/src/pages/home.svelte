<Page name="home">
  <!-- Top Navbar -->
  <Navbar large sliding={false}>
    <NavLeft>
      <Link iconIos="f7:menu" iconAurora="f7:menu" iconMd="material:menu" panelOpen="left" />
    </NavLeft>
    <NavTitle sliding>Cirrus</NavTitle>
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

  <BlockTitle>Navigation</BlockTitle>
  <List>
    <ListItem link='/audio-list/' title="Audio list"/>
  </List>


</Page>

<script>
  import {
    Page,
    Navbar,
    NavLeft,
    NavTitle,
    NavTitleLarge,
    Link,
    BlockTitle,
    List,
    ListItem,
  } from 'framework7-svelte';

</script>