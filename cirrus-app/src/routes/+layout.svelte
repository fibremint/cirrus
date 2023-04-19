<script>
  import { TitleBar } from '$lib/components/titlebar';
	
  import '../app.css';
</script>

<div class="flex flex-col min-h-screen bg-transparent">
	<TitleBar />

	<main class="flex flex-grow flex-col p-4 w-full max-w-5xl my-0 mx-auto box-border bg-transparent">
		<slot />
	</main>
</div>