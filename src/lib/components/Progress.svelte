<script lang="ts">
	export let duration: number;
	export let paused: boolean;
	export let started: boolean;
</script>

<div class="progress-bar" class:paused class:started>
	<div class="progress-value" style:animation-duration={duration + "ms"} />
</div>

<style lang="scss">
	.progress-bar {
		position: relative;

		background-color: var(--c-outline);

		width: 100%;
		height: 1px;

		overflow: visible;

		&.paused .progress-value {
			animation-play-state: paused;
		}

		// reset the animation when starting the timer
		&.started .progress-value {
			animation-name: timer-progress-bar;
		}

		&:not(.started) .progress-value {
			animation-name: none;
		}
	}

	.progress-value {
		--s-height: 0.5rem;

		position: absolute;
		top: calc(var(--s-height) / -2);
		left: 0;
		right: 0;

		background-color: var(--c-tertiary);

		height: var(--s-height);

		animation-name: timer-progress-bar;
		animation-timing-function: linear;
	}

	@keyframes timer-progress-bar {
		from {
			right: 100%;
		}

		to {
			right: 0%;
		}
	}
</style>
