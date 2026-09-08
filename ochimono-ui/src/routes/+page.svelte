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
		<span class="text-sm font-normal text-neutral-500 uppercase">{error || m.ui_loading()}</span>
		{#if error}<button
				class="cursor-pointer border border-neutral-300 px-5 py-2.5"
				onclick={() => location.reload()}>{m.ui_reload()}</button
			>{/if}
	</div>
{/if}
