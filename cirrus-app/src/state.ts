import { writable } from "svelte/store";

import type { AudioTag } from "./types";

export const audioTagsStore = writable<AudioTag[]>([]);
export const selectedAudioTagStore = writable<AudioTag>();