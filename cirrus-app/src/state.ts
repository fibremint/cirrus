import { writable } from "svelte/store";

import type { AudioTag, AudioPlayerContext } from "./types";

export const audioTagsStore = writable<AudioTag[]>([]);
export const selectedAudioTagStore = writable<AudioTag>();
export const audioPlayerContextStore = writable<AudioPlayerContext>();