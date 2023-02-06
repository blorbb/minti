<script lang="ts">
	import TimerList from "$lib/components/TimerList.svelte";
	import { timerControllerList } from "$lib/utils/stores";
</script>

<TimerList />

<button class="add-timer" on:click={timerControllerList.addTimer}> + </button>

<style lang="scss">
	.add-timer {
		--l-size: 2rem;

		background-color: var(--c-tertiary);
		color: var(--c-tertiary-on);

		position: fixed;
		bottom: 2rem;
		right: 2rem;

		width: var(--l-size);
		height: var(--l-size);
		border-radius: 50%;

		font-weight: 900;
	}
</style>
