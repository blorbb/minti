<script lang="ts">
	import { timerControllerList } from "$lib/utils/stores";
	import Timer from "./Timer.svelte";
	import { flip } from "svelte/animate";
	import { getCSSProp } from "$lib/utils/css";
</script>

<div class="c-timer-list">
	{#each $timerControllerList as tc (tc)}
		<div
			class="timer"
			animate:flip={{
				duration: getCSSProp("--t-transition", "time") ?? 100,
			}}
		>
			<Timer {tc} on:remove={() => timerControllerList.removeTimer(tc)} />
		</div>
	{/each}
</div>

<style lang="scss">
	.c-timer-list {
		display: grid;
		gap: var(--l-timer-padding);
		grid-template-columns: repeat(auto-fit, minmax(20rem, 1fr));
		padding: var(--l-timer-padding);
		height: 100%;
		overflow: hidden scroll;
		// scrollbar-gutter: stable both-edges;

		&::-webkit-scrollbar {
			width: 0;
		}

		.timer {
			min-height: 8rem;
			scroll-snap-align: center;
		}
	}
</style>
