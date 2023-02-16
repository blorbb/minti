<script lang="ts">
	import { getCSSProp } from "$lib/utils/css";
	import type { TimerController } from "$lib/utils/timer_controller";
	import { constants } from "$lib/utils/timer_utils";
	import { formatTimeToClock } from "$lib/utils/time_formatter";
	import { parseInput } from "$lib/utils/time_parser";
	import { createEventDispatcher, onDestroy, tick } from "svelte";
	import { scale } from "svelte/transition";
	import Progress from "$lib/components/Progress.svelte";

	export let tc: TimerController;

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
	let duration = 0;

	function updateStatuses() {
		finished = tc.isStopped();
		started = tc.isStarted();
		paused = tc.isPaused();
		running = tc.isRunning();
		duration = tc.getTimerDuration();
	}

	// using interval: NodeJS.Timer raises a linting error
	let interval: ReturnType<typeof setInterval>;

	function startTimerUpdates() {
		interval = setInterval(() => {
			const timeRemaining = tc.getTimeRemaining();
			clockTime = formatTimeToClock(timeRemaining, ["s", "d"], true);
			// remove the last ms, accuracy up to 10ms
			// uncomment if using range ["ms", *]
			// clockTime = clockTime.slice(0, clockTime.length - 1);
		}, INTERVAL_TIME);
	}

	function stopTimerUpdates() {
		clearInterval(interval);
	}

	let input: HTMLInputElement;
	let previousValue = "";
	function submitTime() {
		const time = parseInput(input.value);
		if (time <= 0 || isNaN(time)) return;
		previousValue = input.value;
		tc.reset(time);
		tc.start();

		updateStatuses();
		startTimerUpdates();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.code === "Enter") {
			submitTime();
		}
	}

	const dispatch = createEventDispatcher();

	onDestroy(() => {
		stopTimerUpdates();
	});

	let countdownElem: HTMLElement;
	tc.onFinish(async () => {
		tc.stop();
		stopTimerUpdates();
		updateStatuses();
		clockTime = "0";

		// flash the text
		if (!countdownElem) return;
		countdownElem.classList.add(FINISH_CLASS_NAME);
		for (let i = 0; i < FLASHES * 2; i++) {
			countdownElem.classList.toggle(FINISH_CLASS_NAME);
			await new Promise((resolve) => setTimeout(resolve, FLASH_DURATION));
		}
	});
</script>

<div
	class="c-timer-box"
	transition:scale={{
		duration: getCSSProp("--t-transition", "time") ?? 100,
	}}
>
	<div class="countdown" bind:this={countdownElem}>
		{#if !started}
			<input
				type="text"
				placeholder="Enter Time"
				bind:this={input}
				class:finished
				on:keydown={handleKeydown}
			/>
		{:else}
			{clockTime}
		{/if}
	</div>
	<Progress {duration} {paused} {started} />
	<div class="controls">
		{#if !started}
			<button class="m-primary start" on:click={submitTime}> Start </button>
		{:else}
			<div class="control-left">
				<button
					class="add-time m-light"
					on:click={() => {
						tc.addDuration(constants.MS_IN_MIN);
						updateStatuses();
					}}
				>
					+
				</button>
				<button
					class="subtract-time m-light"
					on:click={() => {
						tc.addDuration(-constants.MS_IN_MIN);
						updateStatuses();
					}}
				>
					-
				</button>
			</div>
			<div class="control-right">
				{#if paused}
					<button
						class="m-primary resume"
						on:click={() => {
							tc.resume();
							updateStatuses();
						}}
					>
						Resume
					</button>
				{:else if running}
					<button
						class="m-primary pause"
						on:click={() => {
							tc.pause();
							updateStatuses();
						}}
					>
						Pause
					</button>
				{:else}
					<button
						class="m-primary reset"
						on:click={async () => {
							tc.reset();
							stopTimerUpdates();
							updateStatuses();
							await tick();
							countdownElem.classList.remove(FINISH_CLASS_NAME);
							input.value = previousValue;
						}}
					>
						Reset
					</button>
				{/if}
			</div>
		{/if}
	</div>
	<button
		class="remove-timer"
		on:click={() => {
			dispatch("remove");
		}}
	>
		×
	</button>
</div>

<style lang="scss">
	.c-timer-box {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: space-evenly;
		position: relative;

		background-color: var(--c-container);
		color: var(--c-secondary-container-on);

		height: 100%;
		border-radius: 0.5rem;

		filter: none;

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
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 3rem;

		// center the start button
		> button.start {
			grid-column: span 2;
		}

		// left/right equidistant from the middle
		> .control-left {
			text-align: right;
		}
	}

	button {
		// transitions
		filter: var(--shadow-drop-2);
		transition-property: filter, outline-width;
		transition-duration: var(--t-transition);

		&:is(:hover, :focus-visible) {
			filter: var(--shadow-drop-3);
		}

		// specific styles
		&.m-primary {
			background-color: var(--c-primary);
			color: var(--c-primary-on);

			// padding: 0.5rem 1rem;
			width: 5rem;
			height: 2rem;
			border-radius: 5rem;
		}

		&.m-light {
			--s-size: 2rem;

			background-color: transparent;
			color: var(--c-text);

			width: var(--s-size);
			height: var(--s-size);
			border-radius: 50%;

			&:is(:hover, :focus-visible) {
				background-color: var(--c-overlay-light);
			}

			&:active {
				background-color: var(--c-overlay-lighter);
			}
		}
	}

	.remove-timer {
		--s-size: 2rem;

		position: absolute;
		top: 1rem;
		right: 1rem;

		background-color: var(--c-error);
		color: var(--c-error-on);

		width: var(--s-size);
		height: var(--s-size);
		padding: 0;
		border-radius: 50%;

		font-weight: 900;
	}
</style>
