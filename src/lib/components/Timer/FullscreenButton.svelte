<script lang="ts">
	import { onDestroy } from "svelte";
	import LightButton from "./LightButton.svelte";

	let isFullscreen = false;
	export let targetElement: HTMLElement;

	function enableFullscreen() {
		if (!document.fullscreenEnabled) return;
		targetElement.requestFullscreen();
	}
	function disableFullscreen() {
		document.exitFullscreen();
	}
	function updateFullscreenStatus() {
		isFullscreen = document.fullscreenElement === targetElement;
	}

	document.addEventListener("fullscreenchange", updateFullscreenStatus);

	onDestroy(() => {
		document.removeEventListener("fullscreenchange", updateFullscreenStatus);
	});
</script>

{#if isFullscreen}
	<LightButton
		icon="ph:corners-in"
		on:click={disableFullscreen}
		tooltipContent="Exit Fullscreen"
	/>
{:else}
	<LightButton
		icon="ph:corners-out"
		on:click={enableFullscreen}
		tooltipContent="Fullscreen"
	/>
{/if}
