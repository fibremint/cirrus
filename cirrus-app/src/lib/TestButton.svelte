<script lang="ts">
  import { emit, listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api'

  let count: number = 0
  const increment = () => {
    count += 1
  }

  function invokeTest() {
    increment();
    invoke('plugin:cirrus|load_audio', { request: 'test-source2.aiff' });
  }
</script>

<button on:click={invokeTest}>
  Do Test
</button>

<style>
  button {
    font-family: inherit;
    font-size: inherit;
    padding: 1em 2em;
    color: #ff3e00;
    background-color: rgba(255, 62, 0, 0.1);
    border-radius: 2em;
    border: 2px solid rgba(255, 62, 0, 0);
    outline: none;
    width: 200px;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
  }

  button:focus {
    border: 2px solid #ff3e00;
  }

  button:active {
    background-color: rgba(255, 62, 0, 0.2);
  }
</style>
