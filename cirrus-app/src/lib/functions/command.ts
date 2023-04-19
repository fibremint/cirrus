import { invoke } from '@tauri-apps/api';

import type { AudioTag, AudioTagRequest } from '../types';

export async function loadAudio(audioTagId: string) {
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

export async function getAudioTags(request: AudioTagRequest): Promise<AudioTag[]> {
  return await invoke('plugin:cirrus|get_audio_tags', request);
}

export async function setPlaybackPosition(positionSec: number) {
  return await invoke('plugin:cirrus|set_playback_position', { playbackPos: positionSec });
}

export async function setListenUpdatedEvents(isListen: boolean) {
  return await invoke('plugin:cirrus|set_listen_updated_events', { isListen: isListen } );
}
