<script lang="ts">
	import { quintOut } from 'svelte/easing';
	import { scale } from 'svelte/transition';
</script>

<div
	class="relative size-[84px] animate-loading-appear motion-reduce:animate-none"
	in:scale={{ start: 0.6, duration: 500, delay: 50, easing: quintOut }}
	out:scale={{ start: 0.6, duration: 250, easing: quintOut }}
	aria-hidden="true"
>
	<div
		class="absolute inset-0 animate-loading-turn rounded-[21px] bg-black/70 motion-reduce:animate-none"
	></div>
	<svg
		class="absolute top-[17px] left-[17px] size-[50px] animate-loading-spin text-white motion-reduce:animate-none"
		viewBox="0 0 60 60"
	>
		<path
			d="M21.45 6.51a25 25 0 1 0 17.1 0"
			fill="none"
			stroke="currentColor"
			stroke-width="7.5"
			stroke-linecap="round"
		/>
	</svg>
</div>
