import { invoke } from '@tauri-apps/api';

export async function loadAudio(audioTagId) {
  return await invoke('plugin:cirrus|load_audio', { audioTagId: audioTagId });
}

export async function playAudio() {
  return await invoke('plugin:cirrus|start_audio');
}

export async function pauseAudio() {
  return await invoke('plugin:cirrus|pause_audio');
}

export async function stopAudio() {
  return await invoke('plugin:cirrus|stop_audio');
}

export async function getAudioTags({ itemsPerPage, currentPage }) {
  return await invoke('plugin:cirrus|get_audio_tags', { itemsPerPage, page: currentPage });
}

export async function setPlaybackPosition(positionSec) {
  return await invoke('plugin:cirrus|set_playback_position', { playbackPos: positionSec });
}

export async function setListenUpdatedEvents(isListen) {
  return await invoke('plugin:cirrus|set_listen_updated_events', { isListen: isListen } );
}