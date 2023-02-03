<script lang="ts">
	import { CSSProps } from "$lib/utils/css";
	import { TimerController } from "$lib/utils/timer_controller";
	import { createEventDispatcher, onDestroy, tick } from "svelte";
	import { scale } from "svelte/transition";

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

	function updateStatuses() {
		finished = tc.isStopped();
		started = tc.isStarted();
		paused = tc.isPaused();
		running = tc.isRunning();
	}

	//#region timer updates
	// using interval: NodeJS.Timer raises a linting error
	let interval: ReturnType<typeof setInterval>;

	function startTimerUpdates() {
		interval = setInterval(() => {
			const timeRemaining = tc.getTimeRemaining();
			clockTime = TimerController.parseToClock(timeRemaining, ["s", "h"], true);
			// remove the last ms, accuracy up to 10ms
			// uncomment if using range ["ms", *]
			// clockTime = clockTime.slice(0, clockTime.length - 1);
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
		tc.reset(time);
		tc.start();
		updateStatuses();
		startTimerUpdates();
	}

	const dispatch = createEventDispatcher();

	onDestroy(() => {
		stopTimerUpdates();
	});
	//#endregion

	let countdownElem: HTMLElement;
	tc.onFinish(async () => {
		tc.stop();
		stopTimerUpdates();
		updateStatuses();
		clockTime = "0";

		// flash the text
		countdownElem.classList.add(FINISH_CLASS_NAME);
		for (let i = 0; i < FLASHES * 2; i++) {
			countdownElem.classList.toggle(FINISH_CLASS_NAME);
			await new Promise((resolve) => setTimeout(resolve, FLASH_DURATION));
		}
	});
</script>

<div
	class="c-timer"
	transition:scale={{
		duration: CSSProps.get("--t-transition", "time") ?? 100,
	}}
>
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
		{#if !started}
			<button class="start" on:click={submitTime}> Start </button>
		{:else}
			<div class="control-left">
				{#if paused}
					<button
						class="resume"
						on:click={() => {
							tc.resume();
							updateStatuses();
						}}
					>
						Resume
					</button>
				{:else if running}
					<button
						class="pause"
						on:click={() => {
							tc.pause();
							updateStatuses();
						}}
					>
						Pause
					</button>
				{:else if finished}
					<button
						class="remove-timer"
						on:click={() => {
							dispatch("remove");
						}}
					>
						Remove
					</button>
				{/if}
			</div>
			<div class="control-right">
				<button
					class="reset"
					on:click={async () => {
						tc.reset();
						stopTimerUpdates();
						updateStatuses();
						await tick();
						countdownElem.classList.remove(FINISH_CLASS_NAME);
						input.value = previousValue.toString();
					}}
				>
					Reset
				</button>
			</div>
		{/if}
	</div>
	<button
		class="remove-timer remove-timer-mini"
		on:click={() => {
			dispatch("remove");
		}}
	>
		-
	</button>
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

		> button.start {
			grid-column: span 2;
		}

		> .control-left {
			text-align: right;
		}
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

	.remove-timer {
		background-color: var(--c-error);
		color: var(--c-error-on);

		&.remove-timer-mini {
			--l-size: 2rem;

			position: absolute;
			top: 1rem;
			right: 1rem;

			width: var(--l-size);
			height: var(--l-size);
			padding: 0;
			border-radius: 50%;

			font-weight: 900;
		}
	}
</style>
