<script lang="ts">
	import Countdown from "$lib/components/Timer/Countdown.svelte";
	import LightButton from "$lib/components/Timer/LightButton.svelte";
	import PrimaryButton from "$lib/components/Timer/PrimaryButton.svelte";
	import Progress from "$lib/components/Timer/Progress.svelte";
	import DurationUpdater from "./Timer/DurationUpdater.svelte";
	import FullscreenButton from "./Timer/FullscreenButton.svelte";

	import { getCSSProp } from "$lib/utils/css";
	import { resetAnimation } from "$lib/utils/misc";
	import { settings, timerControllerList } from "$lib/utils/stores";
	import { ParseError, parseInput } from "$lib/utils/time_parser";
	import type { TimerController } from "$lib/utils/timer_controller";

	import { onDestroy, tick } from "svelte";
	import { scale } from "svelte/transition";

	export let tc: TimerController;

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
				elements.shakeInput();
				return;
			}

			userInput.error.invalid = false;
			userInput.previous = elements.input.value;
			console.log("HERE", $status.started);
			tc.reset(time).start();
		},
		resume: () => tc.resume(),
		pause: () => tc.pause(),
		async reset() {
			tc.reset();
			await tick();
			if (!elements.input) return;
			elements.input.value = userInput.previous;
		},
		duration: {
			add(ms: number) {
				// if already finished, reset to the ms specified
				if ($status.finished) {
					const progressValue =
						elements.timerBox?.querySelector(".progress-value");
					if (!progressValue) return;
					resetAnimation(progressValue as HTMLElement);
					tc.reset(ms).start();
				} else {
					tc.addDuration(ms);
				}

				// jump timer upward
				elements.bumpCountdown("up");
			},
			subtract(ms: number) {
				// clamp so that it stops at 0 if subtracting time
				ms = Math.min(tc.getTimeRemaining(), ms);
				tc.addDuration(-ms);
				// jump timer downward
				elements.bumpCountdown("down");
			},
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
			// play the bump animation
			const bumpDistance =
				$settings.countdownBumpAmount * (direction === "up" ? -1 : 1);
			elements.countdown?.animate(
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
		shakeInput() {
			const shakeDistance = "0.25rem";
			elements.input?.animate(
				[
					{ transform: "translateX(0)" },
					{ transform: `translateX(${shakeDistance})` },
					{ transform: `translateX(-${shakeDistance})` },
					{ transform: `translateX(${shakeDistance})` },
					{ transform: "translateX(0)" },
				],
				{ duration: 200, easing: "ease-out" },
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

	onDestroy(() => {
		tc.display.stopInterval();
		tc.display.stopEndTimeInterval();
	});

	const status = tc.status;
	const duration = tc.duration;
	const timeDisplay = tc.display.timeArray;
	const endTime = tc.display.endTime;
</script>

<div
	class={`c-timer-box`}
	data-paused={$status.paused}
	data-started={$status.started}
	data-finished={$status.finished}
	data-running={$status.running}
	data-settings-progress-bar-type={$settings.progressBarType}
	data-invalid-input={userInput.error.invalid}
	bind:this={elements.timerBox}
	transition:scale={{
		duration: getCSSProp("--t-transition", "time") ?? 100,
	}}
>
	{#if $settings.progressBarType === "background"}
		<Progress
			duration={$duration}
			paused={$status.paused}
			finished={$status.finished}
			started={$status.started}
		/>
	{/if}
	<div class="c-timer-front">
		<div class="extra-status">
			{#if !$status.started && userInput.error.invalid}
				{userInput.error.message}
			{:else if $status.started}
				<iconify-icon inline icon="ph:timer" />
				{$endTime}
			{/if}
		</div>
		<div class="countdown" bind:this={elements.countdown}>
			{#if !$status.started}
				<input
					type="text"
					placeholder="Enter Time"
					bind:this={elements.input}
					class:finished={$status.finished}
					aria-invalid={userInput.error.invalid}
					aria-required
					on:keydown={elements.onInputKeydown}
					on:blur={userInput.updatePrevious}
				/>
			{:else}
				<Countdown times={$timeDisplay} />
			{/if}
		</div>
		{#if $settings.progressBarType === "line"}
			<Progress
				duration={$duration}
				paused={$status.paused}
				finished={$status.finished}
				started={$status.started}
			/>
		{/if}
		<div class="controls">
			{#if !$status.started}
				<div class="control-middle">
					<PrimaryButton
						class="start"
						icon="ph:play-bold"
						on:click={timerTime.start}
						tooltipContent="Start Timer"
					/>
				</div>
			{:else}
				<div class="control-left">
					{#if !$status.finished}
						<DurationUpdater
							type="add"
							on:submitupdate={(event) =>
								timerTime.duration.add(event.detail.duration)}
						/>
						<DurationUpdater
							type="subtract"
							on:submitupdate={(event) =>
								timerTime.duration.subtract(event.detail.duration)}
						/>
					{:else}
						<DurationUpdater
							type="add"
							style="primary"
							on:submitupdate={(event) =>
								timerTime.duration.add(event.detail.duration)}
						/>
					{/if}
				</div>
				<div class="control-right">
					{#if $status.finished}
						<PrimaryButton
							icon="ph:clock-counter-clockwise-bold"
							on:click={timerTime.reset}
							tooltipContent="Reset"
						/>
					{:else}
						{#if $status.paused}
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
		<FullscreenButton
			class="corner-button fullscreen"
			targetElement={elements.timerBox}
		/>
		<LightButton
			class="corner-button remove-timer"
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

		border-radius: inherit;

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
			--s-height: var(--s-button-height);
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
	:global {
		button.corner-button {
			--s-border-radius: var(--l-timer-box__border-radius);

			// inset to be set depending on button
			position: absolute;

			// remove default rounding from .m-light
			border-radius: 0;

			transition-property: background-color, color;
			transition-duration: var(--t-transition);

			&.remove-timer {
				top: 0rem;
				right: 0rem;

				// don't round corner if fullscreen
				border-top-right-radius: inherit;
				border-bottom-left-radius: var(--s-border-radius);

				&:active {
					background-color: var(--c-error);
					color: var(--c-error-on);
					transition: none;
				}
			}

			&.fullscreen {
				bottom: 0rem;
				right: 0rem;

				border-top-left-radius: var(--s-border-radius);
				border-bottom-right-radius: inherit;
			}
		}
	}
</style>
