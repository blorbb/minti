<script lang="ts">
	import { timerControllerList } from "$lib/utils/stores";
	import Timer from "./Timer.svelte";
	import { flip } from "svelte/animate";
	import { CSSProps } from "$lib/utils/css";
</script>

<div class="timer-list">
	{#each $timerControllerList as tc (tc)}
		<div
			class="timer"
			animate:flip={{
				duration: CSSProps.get("--t-transition", "time") ?? 100,
			}}
		>
			<Timer {tc} on:remove={() => timerControllerList.removeTimer(tc)} />
		</div>
	{/each}
</div>

<style lang="scss">
	.timer-list {
		display: flex;
		flex-direction: column;
		padding: 1rem;
		gap: 1rem;
		height: 100%;

		.timer {
			flex-basis: 8rem;
		}
	}
</style>
