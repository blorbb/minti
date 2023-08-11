<script lang="ts">
	import { closest, modulo } from "$lib/utils/misc";
	import { convert } from "$lib/utils/timer_utils";
	import { createEventDispatcher, onDestroy, onMount, tick } from "svelte";
	import tippy, { roundArrow, type Instance, type Props } from "tippy.js";

	// button properties
	let modifierType: "add" | "subtract";
	export { modifierType as type };
	export let style: "light" | "primary" = "light";

	const maybeBold = style == "primary" ? "-bold" : "";
	const iconName =
		modifierType === "add" ? "ph:plus" + maybeBold : "ph:minus" + maybeBold;

	// elements and stuff
	let buttonElem: HTMLButtonElement | undefined;
	let menuElem: HTMLElement | undefined;
	let tippyInstance: Instance<Props>;
	const clickEvent = createEventDispatcher();
	const clickEventName = "submitupdate";

	/**
	 * Sets the `selectedDurationIndex` based on the button clicked.
	 * @param event
	 */
	function setUpdateAmount(event: MouseEvent) {
		const target = event.currentTarget as HTMLButtonElement;
		selectedDurationIndex = +(target.dataset.index ?? 0);
		tippyInstance.hide();
	}

	/**
	 * Dispatches the update event, for the Timer to change its duration.
	 */
	function dispatchUpdateAmount() {
		clickEvent(clickEventName, {
			duration: durations[selectedDurationIndex].ms,
		});
	}

	onMount(() => {
		// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
		tippyInstance = tippy(buttonElem!, {
			content: (ref) =>
				ref.nextElementSibling ?? "Error: Something went very wrong...",
			allowHTML: true,
			interactive: true,
			appendTo: (parent) => parent.closest(".c-timer-box") ?? document.body,
			trigger: "contextmenu",
			offset: [0, 8],
			arrow: roundArrow,
			theme: "overlay-menu",
			onShow() {
				tick().then(() => {
					focusedButtonElem?.focus();
				});
			},
		});
	});

	onDestroy(() => {
		tippyInstance.destroy();
	});

	// implement keyboard navigation
	type Duration = {
		display: string;
		ms: number;
	};

	const durations: Duration[] = [
		{ display: "1d", ms: convert.daysToMs(1) },
		{ display: "1h", ms: convert.hoursToMs(1) },
		{ display: "30m", ms: convert.minsToMs(30) },
		{ display: "10m", ms: convert.minsToMs(10) },
		{ display: "5m", ms: convert.minsToMs(5) },
		{ display: "1m", ms: convert.minsToMs(1) },
		{ display: "30s", ms: convert.secsToMs(30) },
		{ display: "10s", ms: convert.secsToMs(10) },
	];

	// grid information
	const NUM_BUTTONS = 8;
	const DIMENSIONS = {
		rows: 2,
		columns: 4,
	};

	/** Which duration is actually selected. Default is 1m. */
	let selectedDurationIndex = 5;
	let focusedButtonIndex = selectedDurationIndex;

	// reactive elements to ensure focus is always present
	$: focusedButtonElem = menuElem?.children[focusedButtonIndex] as
		| HTMLButtonElement
		| undefined;

	$: if (focusedButtonElem) focusedButtonElem.focus();

	/**
	 * Captures focus within the menu.
	 * @param event
	 */
	function handleMenuKeydown(event: KeyboardEvent) {
		switch (event.code) {
			case "Tab": {
				event.preventDefault();

				const dir = event.shiftKey ? -1 : 1;
				focusedButtonIndex = modulo(focusedButtonIndex + dir, NUM_BUTTONS);
				break;
			}
			case "ArrowLeft":
			case "ArrowRight": {
				// not the same as tabbing
				// cycles around the row if focus is at the last column
				// doesn't go to next row
				event.preventDefault();

				const dir = event.code === "ArrowLeft" ? -1 : 1;
				// 0-indexed
				const currentRow = Math.floor(focusedButtonIndex / DIMENSIONS.columns);
				focusedButtonIndex =
					modulo(focusedButtonIndex + dir, DIMENSIONS.columns) +
					currentRow * DIMENSIONS.columns;
				break;
			}
			case "ArrowUp":
			case "ArrowDown": {
				event.preventDefault();

				const dir = event.code === "ArrowUp" ? -1 : 1;
				focusedButtonIndex = modulo(
					focusedButtonIndex + dir * DIMENSIONS.columns,
					NUM_BUTTONS,
				);
				break;
			}
		}
	}

	/**
	 * Extra listeners to ensure that all keys are captured for the menu.
	 * @param event
	 */
	function handleWindowKeydown(event: KeyboardEvent) {
		if (event.key === "Escape") {
			tippyInstance.hide();
		} else if (
			tippyInstance.state.isVisible &&
			// capture all keyboard events to the menu
			// avoid duplicating the event
			!closest(event.target, ".duration-menu")
		) {
			handleMenuKeydown(event);
		}
	}
</script>

<svelte:window on:keydown={handleWindowKeydown} />

<button
	class={`duration-modifier m-${style}`}
	bind:this={buttonElem}
	on:contextmenu|preventDefault
	on:click={dispatchUpdateAmount}
>
	<iconify-icon inline icon={iconName} />
	<span class="amount">
		{durations[selectedDurationIndex].display}
	</span>
</button>

<div class="duration-menu" bind:this={menuElem} on:keydown={handleMenuKeydown}>
	{#each durations as duration, i}
		<button
			data-index={i}
			data-focused={i === focusedButtonIndex}
			on:focus={() => (focusedButtonIndex = i)}
			on:click={setUpdateAmount}
		>
			{duration.display}
		</button>
	{/each}
</div>

<style lang="scss">
	.duration-modifier {
		&.m-light {
			aspect-ratio: 2/1;

			.amount {
				font-weight: 200;
			}
		}

		.amount {
			font-size: 0.8em;
		}
	}

	// also see css for the `overlay-menu` theme in tippy.scss
	// TODO fix styles, looks bad
	.duration-menu {
		display: grid;
		gap: 0.5rem;
		place-items: center;
		grid-template-columns: repeat(4, 1fr);

		padding: 0.5rem;

		button {
			--s-height: 2rem;

			background-color: var(--c-secondary);
			color: var(--c-secondary-on);
			transition-property: background-color, color;
			transition-duration: var(--t-transition);

			height: var(--s-height);
			aspect-ratio: 2 / 1;
			border-radius: 1rem;

			font-size: var(--l-font-size--small);
			text-align: center;

			// set both focus and focus visible to the same
			&:focus-visible {
				outline-color: transparent;
			}

			&:focus {
				filter: var(--shadow-2--drop);
				background-color: var(--c-primary);
				color: var(--c-primary-on);
			}
		}
	}
</style>
