<script lang="ts">
	import Countdown from "$lib/components/Timer/Countdown.svelte";
	import LightButton from "$lib/components/Timer/LightButton.svelte";
	import PrimaryButton from "$lib/components/Timer/PrimaryButton.svelte";
	import Progress from "$lib/components/Timer/Progress.svelte";

	import { getCSSProp } from "$lib/utils/css";
	import { resetAnimation } from "$lib/utils/misc";
	import { settings, timerControllerList } from "$lib/utils/stores";
	import type { TimerController } from "$lib/utils/timer_controller";
	import {
		constants,
		order,
		type TimeAbbreviations,
	} from "$lib/utils/timer_utils";
	import { formatTimeToStrings } from "$lib/utils/time_formatter";
	import { parseInput } from "$lib/utils/time_parser";

	import { onDestroy, tick } from "svelte";
	import { scale } from "svelte/transition";

	export let tc: TimerController;

	let countdownTimes: [TimeAbbreviations, string][] = [];

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
		// update whenever any status changes
		updateTimer();
	}
	//#endregion

	//#region other statuses/elements
	let fullscreen = document.fullscreenElement !== null;
	let previousValue = "";

	let timerBox: HTMLElement | undefined;
	let countdownElem: HTMLElement | undefined;
	let input: HTMLInputElement | undefined;
	//#endregion

	//#region timer updates
	// using interval: NodeJS.Timer raises a linting error
	let interval: ReturnType<typeof setInterval>;

	function updateTimer() {
		// keep positive so the overtime timer doesn't have
		// negative sign
		const timeRemaining = tc.getTimeRemaining();
		const times = formatTimeToStrings(
			timeRemaining,
			$settings.timerUnitRange,
			$settings.autoTrimTimerDisplay,
		);

		// don't format this as a string as there are different
		// classes for the different parts of the time
		let timeArray = Array.from(order.recordToMap(times)).reverse();

		// check that all digits are 0
		// if so, remove the negative 0
		if (timeArray.every(([, timeStr]) => +timeStr == 0)) {
			// omit the negative 0
			let timeStr = timeArray[0][1];
			if (timeStr[0] === "-") timeStr = timeStr.slice(1);

			timeArray[0][1] = timeStr;
		}

		countdownTimes = timeArray;
	}

	function startTimerUpdates() {
		// run immediately first to avoid blank
		updateTimer();
		interval = setInterval(updateTimer, $settings.timerUpdateInterval);
	}

	function stopTimerUpdates() {
		clearInterval(interval);
	}
	//#endregion

	//#region timer events

	function start() {
		if (!input) return;
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
		if (!input) return;
		input.value = previousValue;
	}

	function bumpAnimation(em: number) {
		return [
			{ transform: "translateY(0px)" },
			{ transform: `translateY(${em}em)` },
			{ transform: "translateY(0px)" },
		];
	}
	const BUMP_OPTIONS: KeyframeAnimationOptions = {
		duration: 100,
		easing: "ease-out",
	};

	async function addDuration(ms: number) {
		// if already finished, reset to the ms specified
		if (finished) {
			const progressValue = timerBox?.querySelector(".progress-value");
			if (!progressValue) return;
			resetAnimation(progressValue as HTMLElement);
			tc.reset(ms).start();
		} else {
			tc.addDuration(ms);
		}

		updateStatuses();
		// jump timer upward
		if (!countdownElem) return;
		countdownElem.animate(bumpAnimation(-0.2), BUMP_OPTIONS);
	}

	function subtractDuration(ms: number) {
		// clamp so that it stops at 0 if subtracting time
		ms = Math.min(tc.getTimeRemaining(), ms);
		tc.addDuration(-ms);
		updateStatuses();
		// jump timer downward
		if (!countdownElem) return;
		countdownElem.animate(bumpAnimation(0.2), BUMP_OPTIONS);
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

	function enableFullscreen() {
		if (!timerBox || !document.fullscreenEnabled) return;
		timerBox.requestFullscreen();
	}

	function disableFullscreen() {
		document.exitFullscreen();
	}

	function updateFullscreen() {
		fullscreen = document.fullscreenElement !== null;
	}

	document.addEventListener("fullscreenchange", updateFullscreen);
	onDestroy(() => {
		stopTimerUpdates();
		document.removeEventListener("fullscreenchange", updateFullscreen);
	});
	//#endregion
</script>

<div
	class={`c-timer-box`}
	data-paused={paused}
	data-started={started}
	data-finished={finished}
	data-running={running}
	data-settings-progress-bar-type={$settings.progressBarType}
	bind:this={timerBox}
	transition:scale={{
		duration: getCSSProp("--t-transition", "time") ?? 100,
	}}
>
	<Progress {duration} {paused} {finished} {started} />
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
				<Countdown times={countdownTimes} />
			{/if}
		</div>
		<div class="controls">
			{#if !started}
				<div class="control-middle">
					<PrimaryButton
						class="start"
						icon="ph:play-bold"
						on:click={start}
						tooltipContent="Start Timer"
					/>
					{#if fullscreen}
						<LightButton
							icon="ph:corners-in"
							on:click={disableFullscreen}
							tooltipContent="Exit Fullscreen"
						/>
					{:else}
						<LightButton
							icon="ph:corners-out"
							on:click={enableFullscreen}
							tooltipContent="Fullscreen"
						/>
					{/if}
				</div>
			{:else}
				<div class="control-left">
					{#if !finished}
						<LightButton
							icon="ph:clock-counter-clockwise"
							on:click={reset}
							tooltipContent="Reset"
						/>
						<LightButton
							icon="ph:plus"
							on:click={() => addDuration(constants.MS_IN_MIN)}
							tooltipContent="Add 1m"
						/>
						<LightButton
							icon="ph:minus"
							on:click={() => subtractDuration(constants.MS_IN_MIN)}
							tooltipContent="Subtract 1m"
						/>
					{:else}
						<PrimaryButton
							icon="ph:plus"
							on:click={() => addDuration(constants.MS_IN_MIN)}
							tooltipContent="Add 1m"
						/>
					{/if}
				</div>
				<div class="control-right">
					{#if finished}
						<PrimaryButton
							icon="ph:clock-counter-clockwise-bold"
							on:click={reset}
							tooltipContent="Reset"
						/>
					{:else if paused}
						<PrimaryButton
							icon="ph:play-bold"
							on:click={resume}
							tooltipContent="Resume"
						/>
					{:else}
						<PrimaryButton
							icon="ph:pause-bold"
							on:click={pause}
							tooltipContent="Pause"
						/>
					{/if}
					{#if fullscreen}
						<LightButton
							icon="ph:corners-in"
							on:click={disableFullscreen}
							tooltipContent="Exit Fullscreen"
						/>
					{:else}
						<LightButton
							icon="ph:corners-out"
							on:click={enableFullscreen}
							tooltipContent="Fullscreen"
						/>
					{/if}
				</div>
			{/if}
		</div>
		<LightButton
			class="remove-timer"
			icon="ph:x"
			on:click={() => {
				timerControllerList.removeTimer(tc);
			}}
			tooltipContent="Remove"
		/>
	</div>
</div>

<style lang="scss">
	.c-timer-box {
		display: flex;
		position: relative;

		height: 100%;
		border-radius: var(--l-timer-box__border-radius);

		overflow: hidden;

		// for the backdrop blur to scale according to the timer size
		container-type: size;

		&[data-settings-progress-bar-type="background"] {
			padding: calc(
				var(--l-progress-bar--bg__padding) +
					var(--l-progress-bar--bg__border-width)
			);
		}

		&[data-finished="true"] .countdown {
			animation: finish-flash 420ms steps(1, end) forwards;
		}

		&[data-paused="true"] .countdown {
			color: var(--c-text--faded);
		}

		&:fullscreen {
			border-radius: 0;
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

		// fallback
		backdrop-filter: blur(1rem);
		backdrop-filter: blur(min(1.5cqw, 1rem));

		// don't transition the backdrop filter
		// makes weird artifacts
		transition-property: background-color;
		transition-duration: var(--t-transition);
		transition-timing-function: ease-in-out;

		&:is(:hover, :focus-within) {
			background-color: hsla(
				var(--p-hsl-timer-front__bgc) / calc(var(--p-a-timer-front__bgc) * 1.02)
			);

			backdrop-filter: blur(1.25rem);
			backdrop-filter: blur(min(2cqw, 1.25rem));
		}
	}

	.countdown {
		height: 2.25rem;

		font-size: 1.5rem;
		font-size: max(1.5rem, 10cqmin);
		font-weight: 700;
		text-align: center;
		// fixed width numbers
		font-variant-numeric: lining-nums tabular-nums;

		input {
			background-color: transparent;

			border: none;
			width: max(15rem, 50%);

			font-weight: normal;
			text-align: center;
			font-variant-numeric: normal;
		}
	}

	.controls {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 3rem;

		> [class^="control"] {
			display: flex;
			align-items: center;
			gap: 0.5rem;
		}

		> .control-middle {
			grid-column: span 2;
		}

		// left/right equidistant from the middle
		> .control-left {
			justify-content: end;
		}
	}

	:global(button.remove-timer.m-light) {
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
