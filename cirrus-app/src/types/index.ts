export type AudioTag = {
  id: string,
  artist: string,
  genre: string,
  title: string,
};

export type AudioPlayerContext = {
  contentLength: number,
  playbackPosition: number,
  remainBuf: number,
}

export type PlaybackPayload = {
  pos: number,
  remainBuf: number,
}