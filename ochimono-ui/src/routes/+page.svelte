<script lang="ts">
	import * as m from '$lib/paraglide/messages';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import LoadingSpinner from '$lib/game-ui/components/loading/LoadingSpinner.svelte';
	let host: HTMLDivElement;
	let ready = $state(false);
	let error = $state('');
	let status = $state<string>(m.ui_loading_game());
	onMount(() => {
		let cancelled = false;
		let client: import('$lib/game-client/GameClient').GameClient | undefined;
		void import('$lib/game-client/GameClient')
			.then(async ({ GameClient }) => {
				if (cancelled) return;
				client = new GameClient(host, (message) => (status = message));
				await client.init();
				if (!cancelled) ready = true;
			})
			.catch((e) => {
				console.error('Game initialization failed', e);
				if (!cancelled) error = m.ui_unable_to_load_the_game_please_reload();
			});
		return () => {
			cancelled = true;
			client?.destroy();
		};
	});
</script>

<svelte:head>
	<title>ochimono</title>
	<meta name="description" content="A falling-block game." />
</svelte:head>

<!-- svelte-ignore a11y_no_noninteractive_tabindex (The canvas application implements its own focus navigation and keyboard input.) -->
<div
	bind:this={host}
	class="game-host fixed inset-0 touch-none overflow-hidden outline-none [&_canvas]:block"
	role="application"
	aria-label={m.ui_ochimono_game()}
	aria-describedby="game-status"
	tabindex="0"
></div>
<p id="game-status" class="sr-only" aria-live="polite">{status}</p>
{#if !ready}
	<div
		class="fixed inset-0 flex flex-col items-center justify-center gap-4 bg-[#f7f7f5]"
		role="status"
		out:fade={{ duration: 250 }}
	>
		<LoadingSpinner />
		{#if error}<span class="text-sm font-normal text-neutral-500">{error}</span>{/if}
		{#if error}<button
				class="cursor-pointer bg-[#6543cb] px-8 py-3 text-sm tracking-widest text-white transition-[transform,background-color] duration-300 ease-out hover:scale-105 hover:bg-[#7957dc] focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-[#6543cb] active:scale-95 motion-reduce:transform-none"
				onclick={() => location.reload()}>{m.ui_reload()}</button
			>{/if}
	</div>
{/if}
