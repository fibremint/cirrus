
import HomePage from '../pages/home.svelte';
import AudioListPage from '../pages/audio-list.svelte';
import NotFoundPage from '../pages/404.svelte';

var routes = [
  {
    path: '/',
    component: HomePage,
  },
  {
    path: '/audio-list/',
    component: AudioListPage,
  },
  {
    path: '(.*)',
    component: NotFoundPage,
  },
];

export default routes;
