<script lang="ts">
	import Timer from "$lib/components/Timer.svelte";
	import { flip } from "svelte/animate";

	let timers: string[] = [];
</script>

<div class="timer-container">
	{#each timers as timerID (timerID)}
		<div
			class="timer"
			animate:flip={{
				duration: 200,
			}}
		>
			<Timer
				on:remove={() => {
					timers = timers.filter((id) => id !== timerID);
				}}
			/>
		</div>
	{/each}
</div>

<button
	class="add-timer"
	on:click={() => (timers = [...timers, crypto.randomUUID()])}
>
	+
</button>

<style lang="scss">
	.timer-container {
		display: flex;
		flex-direction: column;
		padding: 1rem;
		gap: 1rem;
	}

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
