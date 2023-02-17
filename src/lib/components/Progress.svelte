<script lang="ts">
	export let duration: number;
	export let paused: boolean;
	export let started: boolean;
	export let type: "line" | "background";
	export let border: boolean;
</script>

<div
	class={`c-progress-bar`}
	class:style--border={border}
	role="progressbar"
	data-started={started}
	data-paused={paused}
	data-type={type}
>
	<div class="progress-value" style:animation-duration={duration + "ms"} />
</div>

<style lang="scss">
	.c-progress-bar {
		position: absolute;

		background-color: var(--c-container);

		border-radius: inherit;

		&[data-type="background"] {
			inset: 0.2rem;
			// to round the corners but not the progress value in the middle
			overflow: hidden;

			.progress-value {
				top: 0;
				bottom: 0;
				left: 0;
			}
		}

		&[data-type="line"] {
			top: 51%;
			left: 2rem;
			right: 2rem;

			height: 2px;

			z-index: 1;

			.progress-value {
				// ! don't use `inset` as the `right` value
				// needs to be overridden by the `.progress-value`
				// selector below
				top: calc(var(--l-progress-bar--line__height) / -2);
				bottom: calc(var(--l-progress-bar--line__height) / -2);
				left: 0;

				border-radius: inherit;
			}
		}

		&[data-paused="true"] .progress-value {
			animation-play-state: paused;
		}

		// reset the animation when starting the timer
		&[data-started="true"] .progress-value {
			animation-name: timer-progress-bar;
		}

		&[data-started="false"] .progress-value {
			animation-name: none;
		}

		// place box shadow above progress-value
		&.style--border {
			border: 0.2rem solid #393939;
			&::before {
				content: "";

				box-shadow: var(--shadow-2-inset);

				position: absolute;
				inset: 0;
				z-index: 1;

				border-radius: inherit;

				pointer-events: none;
			}
		}
	}

	.progress-value {
		position: absolute;

		right: 100%;

		background-color: var(--c-progress-bar--value__color);

		animation-name: timer-progress-bar;
		animation-timing-function: linear;
		animation-fill-mode: forwards;
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
