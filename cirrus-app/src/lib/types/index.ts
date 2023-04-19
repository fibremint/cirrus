export type AudioTagRequest = {
  itemsPerPage: number,
  page: number,
}

export type AudioTag = {
  artist: string,
  genre: string,
  id: string,
  title: string,
}
