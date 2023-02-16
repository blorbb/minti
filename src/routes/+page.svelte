<script lang="ts">
	import TimerList from "$lib/components/TimerList.svelte";
	import { timerControllerList } from "$lib/utils/stores";
</script>

<svelte:head>
	<title>Timer</title>
</svelte:head>

<TimerList />

<button class="add-timer" on:click={timerControllerList.addTimer}> + </button>

<style lang="scss">
	.add-timer {
		--s-size: 2rem;

		background-color: var(--c-tertiary);
		color: var(--c-tertiary-on);

		position: fixed;
		bottom: 2rem;
		right: 2rem;

		width: var(--s-size);
		height: var(--s-size);
		border-radius: 50%;

		font-weight: 900;
	}
</style>
