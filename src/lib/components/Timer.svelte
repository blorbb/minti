<script lang="ts">
	import Countdown from "$lib/components/Timer/Countdown.svelte";
	import Progress from "$lib/components/Timer/Progress.svelte";

	import { getCSSProp } from "$lib/utils/css";
	import { timerControllerList } from "$lib/utils/stores";
	import type {
		TimeAbbreviations,
		TimerController,
		UnitRange,
	} from "$lib/utils/timer_controller";
	import { constants, order } from "$lib/utils/timer_utils";
	import { formatTimeToStrings } from "$lib/utils/time_formatter";
	import { parseInput } from "$lib/utils/time_parser";

	import { onDestroy, tick } from "svelte";
	import { scale } from "svelte/transition";

	export let tc: TimerController;

	const INTERVAL_TIME = 200;
	const AUTO_TRIM_TIME = true;
	const TIME_UNIT_RANGE: UnitRange = ["s", "d"];
	let progressType: "line" | "background" = "background";

	let countdownTimes: [TimeAbbreviations, string][] = [];
	// ensure that the format is the same
	// e.g. showing 0m 00s if set to ["s", "m"] and auto=false
	const endingTimes = Array.from(
		order.recordToMap(formatTimeToStrings(0, TIME_UNIT_RANGE, AUTO_TRIM_TIME)),
	).reverse();

	//#region statuses
	let finished = false;
	let started = false;
	let paused = false;
	let running = false;
	let duration = 0;

	function updateStatuses() {
		finished = tc.isFinished();
		started = tc.isStarted();
		paused = tc.isPaused();
		running = tc.isRunning();
		duration = tc.getTimerDuration();
	}
	//#endregion

	//#region timer updates
	// using interval: NodeJS.Timer raises a linting error
	let interval: ReturnType<typeof setInterval>;

	function startTimerUpdates() {
		function run() {
			// keep positive so the overtime timer doesn't have
			// negative sign
			const timeRemaining = Math.abs(tc.getTimeRemaining());
			const times = formatTimeToStrings(
				timeRemaining,
				TIME_UNIT_RANGE,
				AUTO_TRIM_TIME,
			);

			countdownTimes = Array.from(order.recordToMap(times)).reverse();

			// remove the last ms, accuracy up to 10ms
			// uncomment if using range ["ms", *]
			// clockTime = clockTime.slice(0, clockTime.length - 1);
		}

		// run immediately first to avoid blank
		run();
		interval = setInterval(run, INTERVAL_TIME);
	}

	function stopTimerUpdates() {
		clearInterval(interval);
	}

	onDestroy(() => {
		stopTimerUpdates();
	});
	//#endregion

	//#region timer events
	let input: HTMLInputElement;
	let previousValue = "";
	function start() {
		const time = parseInput(input.value);
		if (time <= 0 || isNaN(time)) return;
		previousValue = input.value;

		tc.reset(time);
		tc.start();

		updateStatuses();
		startTimerUpdates();
	}

	function resume() {
		tc.resume();
		updateStatuses();
	}

	function pause() {
		tc.pause();
		updateStatuses();
	}

	async function reset() {
		tc.reset();
		stopTimerUpdates();
		updateStatuses();
		await tick();
		input.value = previousValue;
	}

	function addDuration(ms: number) {
		tc.addDuration(ms);
		updateStatuses();
	}

	function subtractDuration(ms: number) {
		// clamp so that it stops at 0 if subtracting time
		ms = Math.min(tc.getTimeRemaining(), ms);
		addDuration(-ms);
	}

	tc.onFinish(() => {
		updateStatuses();
	});
	//#endregion

	//#region misc helpers
	function handleKeydown(event: KeyboardEvent) {
		if (event.code === "Enter") {
			start();
		}
	}
	//#endregion
</script>

<div
	class={`c-timer-box progress--${progressType}`}
	data-paused={paused}
	data-started={started}
	data-finished={finished}
	data-running={running}
	transition:scale={{
		duration: getCSSProp("--t-transition", "time") ?? 100,
	}}
>
	<Progress {duration} {paused} {started} type={progressType} border={false} />
	<div class="c-timer-front">
		<div class="countdown">
			{#if !started}
				<input
					type="text"
					placeholder="Enter Time"
					bind:this={input}
					class:finished
					on:keydown={handleKeydown}
				/>
			{:else if !finished}
				<Countdown times={countdownTimes} />
			{:else}
				<Countdown times={endingTimes} />
			{/if}
		</div>
		<div class="controls">
			{#if !started}
				<button class="m-primary start" on:click={start}>
					<iconify-icon inline icon="ph:play-fill" />
				</button>
			{:else}
				<div class="control-left">
					{#if !finished}
						<button
							class="add-time m-light"
							on:click={() => {
								addDuration(constants.MS_IN_MIN);
							}}
						>
							<iconify-icon inline icon="ph:plus" />
						</button>
						<button
							class="subtract-time m-light"
							on:click={() => {
								subtractDuration(constants.MS_IN_MIN);
							}}
						>
							<iconify-icon inline icon="ph:minus" />
						</button>
						<button class="reset-timer m-light" on:click={reset}>
							<iconify-icon inline icon="ph:clock-counter-clockwise" />
						</button>
					{:else}
						<span class="overtime-timer">
							<iconify-icon inline icon="ph:timer-bold" />
							<Countdown times={countdownTimes} />
						</span>
					{/if}
				</div>
				<div class="control-right">
					{#if paused}
						<button class="m-primary resume" on:click={resume}>
							<iconify-icon inline icon="ph:play-fill" />
						</button>
					{:else if running && !finished}
						<button class="m-primary pause" on:click={pause}>
							<iconify-icon inline icon="ph:pause-fill" />
						</button>
					{:else}
						<button class="m-primary reset" on:click={reset}>
							<iconify-icon inline icon="ph:clock-counter-clockwise-bold" />
						</button>
					{/if}
				</div>
			{/if}
		</div>
		<button
			class="remove-timer m-light"
			on:click={() => {
				timerControllerList.removeTimer(tc);
			}}
		>
			<iconify-icon inline icon="ph:x" />
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

		&[data-finished="true"] .countdown {
			animation: finish-flash 420ms steps(1, end) forwards;
		}
	}

	@keyframes finish-flash {
		0%,
		50%,
		100% {
			color: var(--c-timer--countdown__finish-color);
		}

		25%,
		75% {
			color: var(--c-text);
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

	.countdown {
		height: 2.25rem;

		font-size: 1.5rem;
		font-weight: 700;
		text-align: center;
		// fixed width numbers
		font-variant-numeric: lining-nums tabular-nums;

		input {
			background-color: transparent;

			border: none;
			width: clamp(15rem, 50%, 25rem);

			font-weight: normal;
			text-align: center;
			font-variant-numeric: normal;
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

		> :is(.control-left, .control-right) {
			display: flex;
			align-items: center;
		}

		// left/right equidistant from the middle
		> .control-left {
			justify-content: end;
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

		border-radius: 0 0.5rem;

		transition-property: background-color, color;
		transition-duration: var(--t-transition);

		&:active {
			background-color: var(--c-error);
			color: var(--c-error-on);
			transition: none;
		}
	}

	.overtime-timer {
		color: var(--c-timer--countdown__finish-color);
		font-variant-numeric: lining-nums tabular-nums;
	}
</style>
