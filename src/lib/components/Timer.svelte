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
		update() {
			timerStatus.finished = tc.isFinished();
			timerStatus.started = tc.isStarted();
			timerStatus.paused = tc.isPaused();
			timerStatus.running = tc.isRunning();
			timerStatus.duration = tc.getTimerDuration();
			// update whenever any status changes
			timerDisplay.update();
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
		},
		pause() {
			tc.pause();
			timerStatus.update();
		},
		async reset() {
			tc.reset();
			timerDisplay.stopInterval();
			timerStatus.update();
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
		endTime: "",
		endTimeFormat: new Intl.DateTimeFormat(undefined, {
			hour: "numeric",
			minute: "numeric",
		}).format,
		updateInterval: undefined as Maybe<NodeJS.Timer>,
		update() {
			// end time
			timerDisplay.endTime = formatRelativeTime(tc.getTimeRemaining());

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
			if (timerDisplay.updateInterval) timerDisplay.stopInterval();
			timerDisplay.update();
			timerDisplay.updateInterval = setInterval(
				timerDisplay.update,
				$settings.timerUpdateInterval,
			);
		},
		stopInterval() {
			clearInterval(timerDisplay.updateInterval);
			timerDisplay.updateInterval = undefined;
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
		bumpCountdown(direction: "up" | "down") {
			if (!elements.countdown) return;
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
					<PrimaryButton
						class="start"
						icon="ph:play-bold"
						on:click={timerTime.start}
						tooltipContent="Start Timer"
					/>
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
				</div>
			{:else}
				<div class="control-left">
					{#if !timerStatus.finished}
						<LightButton
							icon="ph:clock-counter-clockwise"
							on:click={timerTime.reset}
							tooltipContent="Reset"
						/>
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
					{:else if timerStatus.paused}
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

		// backdrop blur and font size scale according to container size
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

		&[data-invalid-input="true"] .extra-status {
			color: var(--c-error);
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
		justify-content: center;
		gap: max(0.25rem, 3cqh);

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

	.extra-status {
		color: var(--c-timer--countdown__finish-color);
		font-size: var(--l-font-size--small);
		text-align: center;
	}

	.countdown {
		font-size: 1.5rem;
		font-size: clamp(1.5rem, calc(10cqmin + 0.5rem), 4rem);
		font-weight: 700;
		text-align: center;
		// fixed width numbers
		font-variant-numeric: lining-nums tabular-nums;

		input {
			background-color: transparent;

			border: none;
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

	.controls {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 3rem;

		:global(button) {
			--s-size: clamp(1.5rem, 5cqh + 1rem, 2rem);
		}

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
</style>
