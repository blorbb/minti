<script lang="ts">
	import Countdown from "$lib/components/Timer/Countdown.svelte";
	import LightButton from "$lib/components/Timer/LightButton.svelte";
	import PrimaryButton from "$lib/components/Timer/PrimaryButton.svelte";
	import Progress from "$lib/components/Timer/Progress.svelte";

	import { getCSSProp } from "$lib/utils/css";
	import { formatRelativeTime } from "$lib/utils/date";
	import { resetAnimation } from "$lib/utils/misc";
	import { settings, timerControllerList } from "$lib/utils/stores";
	import type { TimerController } from "$lib/utils/timer_controller";
	import {
		constants,
		order,
		type TimeAbbreviations,
	} from "$lib/utils/timer_utils";
	import { formatTimeToStrings } from "$lib/utils/time_formatter";
	import { ParseError, parseInput } from "$lib/utils/time_parser";

	import { onDestroy, tick } from "svelte";
	import { scale } from "svelte/transition";

	export let tc: TimerController;

	const timerStatus = {
		finished: false,
		started: false,
		paused: false,
		running: false,
		duration: 0,
		/**
		 * Updates all the statuses and recalculates the countdown and end times.
		 */
		update() {
			timerStatus.finished = tc.isFinished();
			// unpause the timer if its finished, so that
			// the overtime timer continues
			if (timerStatus.finished) {
				tc.resume();
				timerDisplay.startInterval();
			}

			timerStatus.started = tc.isStarted();
			timerStatus.paused = tc.isPaused();
			timerStatus.running = tc.isRunning();
			timerStatus.duration = tc.getTimerDuration();
			// update whenever any status changes
			timerDisplay.update();
			timerDisplay.updateEndTime();
		},
	};
	tc.onFinish(timerStatus.update);

	const timerTime = {
		async start() {
			if (!elements.input) return;

			let time: number;
			try {
				time = parseInput(elements.input.value);
				if (time <= 0) throw new ParseError("Time must be positive");
				if (isNaN(time)) throw new ParseError("Invalid input");
			} catch (err) {
				if (!(err instanceof ParseError)) throw err;

				userInput.error.message = err.message;
				userInput.error.invalid = true;
				return;
			}

			userInput.error.invalid = false;
			userInput.previous = elements.input.value;
			tc.reset(time).start();
			timerStatus.update();
			timerDisplay.startInterval();
		},
		resume() {
			tc.resume();
			timerStatus.update();
			timerDisplay.startInterval();
		},
		pause() {
			tc.pause();
			timerStatus.update();
			timerDisplay.stopInterval();
		},
		async reset() {
			tc.reset();
			timerStatus.update();
			timerDisplay.stopInterval();
			await tick();
			if (!elements.input) return;
			elements.input.value = userInput.previous;
		},
		duration: {
			async add(ms: number) {
				// in case used the wrong function
				if (ms < 0) {
					console.warn(
						`Warning: Used a negative time (${ms}) in duration.add function. Calling duration.subtract instead. Stack trace below.`,
					);
					console.trace();
					timerTime.duration.subtract(-ms);
					return;
				}

				// if already finished, reset to the ms specified
				if (timerStatus.finished) {
					const progressValue =
						elements.timerBox?.querySelector(".progress-value");
					if (!progressValue) return;
					resetAnimation(progressValue as HTMLElement);
					tc.reset(ms).start();
				} else {
					tc.addDuration(ms);
				}

				timerStatus.update();
				// jump timer upward
				elements.bumpCountdown("up");
			},
			subtract(ms: number) {
				// in case used the wrong function
				if (ms < 0) {
					console.warn(
						`Warning: Used a negative time (${ms}) in duration.subtract function. Calling duration.add instead. Stack trace below.`,
					);
					console.trace();
					timerTime.duration.add(-ms);
					return;
				}

				// clamp so that it stops at 0 if subtracting time
				ms = Math.min(tc.getTimeRemaining(), ms);
				tc.addDuration(-ms);
				timerStatus.update();
				// jump timer downward
				elements.bumpCountdown("down");
			},
		},
	};

	const timerDisplay = {
		timeArray: [] as [TimeAbbreviations, string][],
		_updateInterval: undefined as Maybe<NodeJS.Timer>,
		update() {
			// countdown
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

			timerDisplay.timeArray = timeArray;
		},
		startInterval() {
			if (timerDisplay._updateInterval) timerDisplay.stopInterval();
			// status should be updated which already calls an update
			// shouldn't need, but uncomment if needed
			// timerDisplay.update();
			timerDisplay._updateInterval = setInterval(
				timerDisplay.update,
				$settings.timerUpdateInterval,
			);

			timerDisplay.stopEndTimeInterval();
		},
		stopInterval() {
			clearInterval(timerDisplay._updateInterval);
			timerDisplay._updateInterval = undefined;

			timerDisplay._startEndTimeInterval();
		},
		// end time
		// when the timer starts counting down, stop refreshing the end time
		// when time is paused, refresh end times
		endTime: "",
		_endTimeUpdateInterval: undefined as Maybe<NodeJS.Timer>,
		updateEndTime() {
			timerDisplay.endTime = formatRelativeTime(tc.getTimeRemaining());
		},
		_startEndTimeInterval() {
			if (timerDisplay._endTimeUpdateInterval)
				timerDisplay.stopEndTimeInterval();
			// status should be updated which already calls an update
			// shouldn't need, but uncomment if needed
			// timerDisplay.updateEndTime();
			timerDisplay._endTimeUpdateInterval = setInterval(
				timerDisplay.updateEndTime,
				2000,
			);
		},
		stopEndTimeInterval() {
			clearInterval(timerDisplay._endTimeUpdateInterval);
			timerDisplay._endTimeUpdateInterval = undefined;
		},
	};

	type Maybe<T> = T | undefined;
	const elements = {
		timerBox: undefined as Maybe<HTMLElement>,
		countdown: undefined as Maybe<HTMLElement>,
		input: undefined as Maybe<HTMLInputElement>,
		onInputKeydown(event: KeyboardEvent) {
			if (event.code === "Enter") {
				timerTime.start();
			} else if (event.code === "Escape") {
				if (!elements.input) return;
				elements.input.value = userInput.previous;
				elements.input.blur();
			}
		},
		async bumpCountdown(direction: "up" | "down") {
			if (!elements.countdown) return;

			// play the bump animation
			const bumpDistance =
				$settings.countdownBumpAmount * (direction === "up" ? -1 : 1);
			elements.countdown.animate(
				[
					{ transform: "translateY(0px)" },
					{ transform: `translateY(${bumpDistance}em)` },
					{ transform: "translateY(0px)" },
				],
				{
					duration: 100,
					easing: "ease-out",
				},
			);
		},
	};

	const userInput = {
		previous: "",
		error: {
			message: "",
			invalid: false,
		},
		updatePrevious() {
			if (elements.input) userInput.previous = elements.input.value;
		},
	};

	const fullscreen = {
		// if timerBoxElem is undefined, isFullscreen === false
		isFullscreen: document.fullscreenElement === elements.timerBox,
		enable() {
			if (!elements.timerBox || !document.fullscreenEnabled) return;
			elements.timerBox.requestFullscreen();
		},
		disable() {
			document.exitFullscreen();
		},
		updateStatus() {
			fullscreen.isFullscreen =
				document.fullscreenElement === elements.timerBox;
		},
	};

	document.addEventListener("fullscreenchange", fullscreen.updateStatus);
	onDestroy(() => {
		timerDisplay.stopInterval();
		timerDisplay.stopEndTimeInterval();
		document.removeEventListener("fullscreenchange", fullscreen.updateStatus);
	});
</script>

<div
	class={`c-timer-box`}
	data-paused={timerStatus.paused}
	data-started={timerStatus.started}
	data-finished={timerStatus.finished}
	data-running={timerStatus.running}
	data-settings-progress-bar-type={$settings.progressBarType}
	data-invalid-input={userInput.error.invalid}
	bind:this={elements.timerBox}
	transition:scale={{
		duration: getCSSProp("--t-transition", "time") ?? 100,
	}}
>
	<Progress
		duration={timerStatus.duration}
		paused={timerStatus.paused}
		finished={timerStatus.finished}
		started={timerStatus.started}
	/>
	<div class="c-timer-front">
		<div class="extra-status">
			{#if !timerStatus.started && userInput.error.invalid}
				{userInput.error.message}
			{:else if timerStatus.started}
				<iconify-icon inline icon="ph:timer" />
				{timerDisplay.endTime}
			{/if}
			&ZeroWidthSpace; <!-- keep the box -->
		</div>
		<div class="countdown" bind:this={elements.countdown}>
			{#if !timerStatus.started}
				<input
					type="text"
					placeholder="Enter Time"
					bind:this={elements.input}
					class:finished={timerStatus.finished}
					aria-invalid={userInput.error.invalid}
					aria-required
					on:keydown={elements.onInputKeydown}
					on:blur={userInput.updatePrevious}
				/>
			{:else}
				<Countdown times={timerDisplay.timeArray} />
			{/if}
		</div>
		<div class="controls">
			{#if !timerStatus.started}
				<div class="control-middle">
					{#if fullscreen.isFullscreen}
						<LightButton
							icon="ph:corners-in"
							on:click={fullscreen.disable}
							tooltipContent="Exit Fullscreen"
						/>
					{:else}
						<LightButton
							icon="ph:corners-out"
							on:click={fullscreen.enable}
							tooltipContent="Fullscreen"
						/>
					{/if}
					<PrimaryButton
						class="start"
						icon="ph:play-bold"
						on:click={timerTime.start}
						tooltipContent="Start Timer"
					/>
				</div>
			{:else}
				<div class="control-left">
					{#if !timerStatus.finished}
						{#if fullscreen.isFullscreen}
							<LightButton
								icon="ph:corners-in"
								on:click={fullscreen.disable}
								tooltipContent="Exit Fullscreen"
							/>
						{:else}
							<LightButton
								icon="ph:corners-out"
								on:click={fullscreen.enable}
								tooltipContent="Fullscreen"
							/>
						{/if}
						<LightButton
							icon="ph:plus"
							on:click={() => timerTime.duration.add(constants.MS_IN_MIN)}
							tooltipContent="Add 1m"
						/>
						<LightButton
							icon="ph:minus"
							on:click={() => timerTime.duration.subtract(constants.MS_IN_MIN)}
							tooltipContent="Subtract 1m"
						/>
					{:else}
						<PrimaryButton
							icon="ph:plus"
							on:click={() => timerTime.duration.add(constants.MS_IN_MIN)}
							tooltipContent="Add 1m"
						/>
					{/if}
				</div>
				<div class="control-right">
					{#if timerStatus.finished}
						<PrimaryButton
							icon="ph:clock-counter-clockwise-bold"
							on:click={timerTime.reset}
							tooltipContent="Reset"
						/>
					{:else}
						{#if timerStatus.paused}
							<PrimaryButton
								icon="ph:play-bold"
								on:click={timerTime.resume}
								tooltipContent="Resume"
							/>
						{:else}
							<PrimaryButton
								icon="ph:pause-bold"
								on:click={timerTime.pause}
								tooltipContent="Pause"
							/>
						{/if}
						<LightButton
							icon="ph:clock-counter-clockwise"
							on:click={timerTime.reset}
							tooltipContent="Reset"
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
	///
	/// `.c-timer-box` contains all the statuses.
	///
	/// Styles to fill the slot given by TimerList
	/// and providing context for other elements
	/// (positioning and container queries).
	///
	.c-timer-box {
		display: flex;
		position: relative;

		height: 100%;
		border-radius: var(--l-timer-box__border-radius);

		overflow: hidden;
		container-type: size;

		///
		/// Other box styles:
		/// - add padding if using background style progress bar
		/// - remove border radius if fullscreen so it fills up
		///   the whole viewport
		///
		&[data-settings-progress-bar-type="background"] {
			padding: calc(
				var(--l-progress-bar--bg__padding) +
					var(--l-progress-bar--bg__border-width)
			);
		}

		&:fullscreen {
			border-radius: 0;
		}
	}

	///
	/// The lighter grey box that contains all the elements
	/// on the timer. Does not contain the progress bar.
	///
	/// Custom properties are defined here to be able to
	/// use the container query lengths.
	///
	.c-timer-front {
		--s-status-font-size: clamp(var(--l-font-size--small), 0.3rem + 3cqh, 1rem);
		--s-countdown-font-size: clamp(1.5rem, calc(10cqmin + 0.5rem), 4rem);
		--s-flex-gap: max(0.25rem, 3cqh);
		--s-hsl-front: var(--p-hsl-timer-front__bgc);
		--s-a-front: var(--p-a-timer-front__bgc);
		--s-button-height: clamp(1.25rem, 5cqh + 1rem, 2rem);

		flex-grow: 1;

		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--s-flex-gap);

		position: relative;

		background-color: hsla(var(--s-hsl-front) / var(--s-a-front));
		color: var(--c-secondary-container-on);

		border-radius: var(--l-timer-box__border-radius);

		backdrop-filter: blur(min(1.5cqw, 1rem));

		// don't transition the backdrop filter
		// makes weird artifacts
		transition: background-color var(--t-transition) ease-in-out;

		&:is(:hover, :focus-within) {
			background-color: hsla(
				var(--s-hsl-front) / calc(var(--s-a-front) * 1.02)
			);
			backdrop-filter: blur(min(2cqw, 1.25rem));
		}
	}

	///
	/// Part above the timer countdown. Used to show
	/// the end time and any input errors.
	///
	.extra-status {
		--s-block-height: calc(var(--s-status-font-size) * var(--line-height));

		// 0px so that the countdown and controls are
		// centered when there is no error or not started
		height: 0px;

		color: var(--c-timer--countdown__finish-color);
		font-size: var(--s-status-font-size);
		text-align: center;

		transition: height var(--t-transition);

		// make the text slide out when appearing instead
		// of appearing then moving
		overflow: hidden;
	}

	[data-started="true"] .extra-status {
		height: var(--s-block-height);
	}

	[data-invalid-input="true"] .extra-status {
		height: var(--s-block-height);
		color: var(--c-error);
	}

	///
	/// Main timer. Contains the input as well as the countdown.
	///
	.countdown {
		font-size: var(--s-countdown-font-size);
		font-weight: 700;
		text-align: center;
		// fixed width numbers
		font-variant-numeric: lining-nums tabular-nums;

		// for pause/unpause. doesn't affect the finish-flash animation
		transition: color var(--t-transition);

		input {
			border-radius: 0.5rem;
			width: max(5rem, 70%);

			font-weight: normal;
			text-align: center;
			font-variant-numeric: normal;

			&[aria-invalid="true"] {
				outline: 3px solid var(--c-error);
			}
		}
	}

	[data-paused="true"] .countdown {
		color: var(--c-text--faded);
	}

	///
	/// finish-flash animation
	/// flash red without transition/fade
	///
	[data-finished="true"] .countdown {
		animation: finish-flash 420ms steps(1, end) forwards;
	}

	// currently at 3 flashes
	// add more with more percentages
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

	///
	/// Buttons below the timer countdown.
	/// Split into left/right when the timer has started,
	/// otherwise it is just one middle part.
	///
	.controls {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 3rem;

		// add padding equal to the one created by the line height
		// from the extra status
		padding-block: calc(
			(var(--s-status-font-size) * (var(--line-height) - 1)) / 2
		);

		:global(button) {
			--s-size: var(--s-button-height);
		}

		> * {
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

	///
	/// Remove timer button.
	///
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
</style>
