
<App { ...f7params } >

 
  <!-- Your main view, should have "view-main" class -->
  <View main class="safe-areas dark color-deeppurple" url="/" />

</App>

<script>
  import { onMount } from 'svelte';

  import {
    f7ready,
    App,
    View,
  } from 'framework7-svelte';

  import routes from '../js/routes';
 
  // Framework7 Parameters
  let f7params = {
    name: 'Cirrus', // App name
    theme: 'auto', // Automatic theme detection

    // App store
    // store: store,
    // App routes
    routes: routes,
  };

  onMount(() => {
    f7ready(() => {


      // Call F7 APIs here
    });
  })

  // onDestroy(() => {
  //   console.log('destroy');
  //   invoke('plugin:cirrus|stop_audio');
  // })
</script>