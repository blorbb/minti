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

	const INTERVAL_TIME = 200;
	const FINISH_CLASS_NAME = "finished";
	const FLASHES = 3;
	const FLASH_DURATION = 100;
	let progressType: "line" | "background" = "background";

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
	class={`c-timer-box progress--${progressType}`}
	transition:scale={{
		duration: getCSSProp("--t-transition", "time") ?? 100,
	}}
>
	<Progress {duration} {paused} {started} type={progressType} border={false} />
	<div class="c-timer-front">
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
			class="remove-timer m-light"
			on:click={() => {
				dispatch("remove");
			}}
		>
			×
		</button>
	</div>
</div>

<style lang="scss">
	.c-timer-box {
		position: relative;
		display: flex;
		height: 100%;
		border-radius: var(--l-timer-box__border-radius);
		overflow: hidden;

		&.progress--background {
			padding: var(--l-progress-bar--bg__padding);
		}
	}

	.c-timer-front {
		flex-grow: 1;

		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: space-evenly;
		position: relative;

		background-color: hsla(
			var(--p-hsl-timer-front__bgc) / var(--p-a-timer-front__bgc)
		);
		color: var(--c-secondary-container-on);

		border-radius: var(--l-timer-box__border-radius);

		backdrop-filter: blur(1rem);

		// don't transition the backdrop filter
		// makes weird artifacts
		transition-property: background-color;
		transition-duration: var(--t-transition);
		transition-timing-function: ease-in-out;

		&:is(:hover, :focus-within) {
			background-color: hsla(
				var(--p-hsl-timer-front__bgc) / calc(var(--p-a-timer-front__bgc) + 0.02)
			);

			backdrop-filter: blur(1.5rem);
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

		height: 1.5rem;

		font-size: 1.5rem;
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
				background-color: var(--c-overlay-lightest);
			}
		}
	}

	button.remove-timer {
		position: absolute;
		top: 0rem;
		right: 0rem;

		border-radius: 0 0 0 0.5rem;
	}
</style>
