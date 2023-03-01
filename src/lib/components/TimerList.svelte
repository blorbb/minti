<script lang="ts">
	import { timerControllerList } from "$lib/utils/stores";
	import Timer from "./Timer.svelte";
	import { flip } from "svelte/animate";
	import { getCSSProp } from "$lib/utils/css";
</script>

<div class="c-timer-list">
	{#each $timerControllerList as tc (tc)}
		<div
			class="timer-container"
			animate:flip={{
				duration: getCSSProp("--t-transition", "time") ?? 100,
			}}
		>
			<Timer {tc} />
		</div>
	{/each}
</div>

<style lang="scss">
	.c-timer-list {
		--s-min-timer-width: 22rem;

		display: grid;
		gap: var(--l-timer-list__padding);
		grid-template-columns: repeat(
			auto-fit,
			minmax(min(var(--s-min-timer-width), 100%), 1fr)
		);
		padding: var(--l-timer-list__padding);
		min-height: 100%;

		&::-webkit-scrollbar {
			width: 0;
		}

		.timer-container {
			min-height: 8rem;
		}
	}
</style>
