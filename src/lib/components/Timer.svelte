<script lang="ts">
	import { Timer } from "$lib/utils/timer";
	import { onDestroy, onMount } from "svelte";
	import { tick } from "svelte";

	const timer = new Timer(0);

	const INTERVAL_TIME = 5;
	const FINISH_CLASS_NAME = "finished";
	const FLASHES = 3;
	const FLASH_DURATION = 100;

	let clockTime = "";

	// statuses
	let finished = false;
	let started = false;
	let paused = false;
	let running = false;

	function updateStatuses() {
		finished = timer.isStopped();
		started = timer.isStarted();
		paused = timer.isPaused();
		running = timer.isRunning();
	}

	//#region timer updates
	// using interval: NodeJS.Timer raises a linting error
	let interval: ReturnType<typeof setInterval>;

	function startTimerUpdates() {
		interval = setInterval(() => {
			const timeRemaining = timer.getTimeRemaining();
			clockTime = Timer.parseToClock(timeRemaining);
			// remove the last ms, accuracy up to 10ms
			clockTime = clockTime.slice(0, clockTime.length - 1);
		}, INTERVAL_TIME);
	}

	function stopTimerUpdates() {
		clearInterval(interval);
	}

	let input: HTMLInputElement;
	let previousValue = 0;
	function submitTime() {
		const time = +input.value;
		if (time <= 0 || isNaN(time)) return;
		previousValue = time;
		timer.reset(time);
		timer.start();
		updateStatuses();
		startTimerUpdates();
	}

	onMount(() => {
		// input.value = previousValue.toString();
	});

	onDestroy(() => {
		stopTimerUpdates();
	});
	//#endregion

	let countdownElem: HTMLElement;
	timer.onFinish(async () => {
		timer.stop();
		stopTimerUpdates();
		updateStatuses();
		clockTime = "0:00:00.00";

		// flash the text
		countdownElem.classList.add(FINISH_CLASS_NAME);
		for (let i = 0; i < FLASHES * 2; i++) {
			countdownElem.classList.toggle(FINISH_CLASS_NAME);
			await new Promise((resolve) => setTimeout(resolve, FLASH_DURATION));
		}
	});
</script>

<div class="c-timer">
	<div class="countdown" bind:this={countdownElem}>
		{#if !started}
			<input
				type="text"
				placeholder="Enter Time (ms)"
				bind:this={input}
				class:finished
			/>
		{:else}
			{clockTime}
		{/if}
	</div>
	<div class="controls">
		{#if paused}
			<button
				class="resume"
				on:click={() => {
					timer.resume();
					updateStatuses();
				}}
			>
				Resume
			</button>
		{/if}
		{#if running}
			<button
				class="pause"
				on:click={() => {
					timer.pause();
					updateStatuses();
				}}
			>
				Pause
			</button>
		{/if}
		{#if !started}
			<button class="start" on:click={submitTime}> Start </button>
		{:else}
			<button
				class="reset"
				on:click={async () => {
					timer.reset();
					stopTimerUpdates();
					updateStatuses();
					await tick();
					countdownElem.classList.remove(FINISH_CLASS_NAME);
					input.value = previousValue.toString();
				}}
			>
				Reset
			</button>
		{/if}
	</div>
</div>

<style lang="scss">
	.c-timer {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: space-evenly;
		position: relative;

		background-color: var(--c-container);
		color: var(--c-secondary-container-on);
		// https://stackoverflow.com/questions/15064940
		// level 1 and level 2 shadows
		filter: none;

		height: 8rem;
		border-radius: 0.5rem;

		// will-change: filter;
		transition-property: filter, background-color;
		transition-duration: var(--t-transition);
		transition-timing-function: ease-in-out;

		&:is(:hover, :focus-within) {
			background-color: var(--c-container-up);
			filter: var(--shadow-drop-2);
		}
	}

	input {
		background-color: transparent;

		border: none;
		width: clamp(15rem, 50%, 25rem);

		font-weight: normal;
		text-align: center;
		font-feature-settings: var(--default-font-feature-settings);
	}

	.countdown {
		display: flex;
		justify-content: center;

		height: 3rem;

		font-size: 2rem;
		font-weight: 700;
		// fixed width numbers
		font-feature-settings: "tnum", var(--default-font-feature-settings);

		&:global(.finished) {
			color: rgb(235, 86, 59);
		}
	}

	.controls {
		display: flex;
		justify-content: center;
		gap: 3rem;

		width: 50%;
	}

	button {
		background-color: var(--c-primary);
		color: var(--c-primary-on);

		padding: 0.5rem 1rem;
		border-radius: 5rem;

		filter: var(--shadow-drop-2);
		transition-property: filter, outline-width;
		transition-duration: var(--t-transition);

		&:is(:hover, :focus-visible) {
			filter: var(--shadow-drop-3);
		}
	}
</style>
