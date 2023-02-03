<script lang="ts">
	import TimerList from "$lib/components/TimerList.svelte";
	import { timerControllerList } from "$lib/utils/stores";
</script>

<TimerList />

<button class="add-timer" on:click={timerControllerList.addTimer}> + </button>

<style lang="scss">
	.add-timer {
		--l-size: 3rem;

		background-color: var(--c-tertiary);
		color: var(--c-tertiary-on);

		position: fixed;
		bottom: 2rem;
		left: 50%;
		transform: translateX(-50%);

		width: var(--l-size);
		height: var(--l-size);
		border-radius: 50%;

		font-weight: 900;
		font-size: 1.5rem;
	}
</style>
