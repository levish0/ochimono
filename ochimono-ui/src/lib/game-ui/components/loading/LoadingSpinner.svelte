<script lang="ts">
	import { quintOut } from 'svelte/easing';
	import { scale } from 'svelte/transition';
</script>

<div
	class="relative size-[60px] animate-loading-appear overflow-hidden rounded-[15px] bg-black/70 motion-reduce:animate-none"
	in:scale={{ start: 0.6, duration: 500, delay: 50, easing: quintOut }}
	out:scale={{ start: 0.6, duration: 250, easing: quintOut }}
	aria-hidden="true"
>
	<svg class="absolute inset-0 size-full text-black/20" viewBox="0 0 60 60">
		<g
			class="animate-loading-pattern motion-reduce:animate-none"
			fill="currentColor"
			stroke="currentColor"
			stroke-width="0.6"
		>
			<path d="M-8 36 8 8 24 36Z M20 62 42 24 64 62Z M-8 96 8 68 24 96Z M20 122 42 84 64 122Z" />
			<path
				d="M24 20 36 0 48 20Z M-6 64 12 34 30 64Z M24 80 36 60 48 80Z M-6 124 12 94 30 124Z"
				fill="none"
			/>
		</g>
	</svg>
	<svg
		class="absolute top-3 left-3 size-9 animate-loading-spin text-white motion-reduce:animate-none"
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
