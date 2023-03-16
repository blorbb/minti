<script lang="ts">
	import { convert } from "$lib/utils/timer_utils";
	import { createEventDispatcher, onDestroy, onMount } from "svelte";
	import tippy, { roundArrow, type Instance, type Props } from "tippy.js";

	let modifierType: "add" | "subtract";
	export { modifierType as type };
	export let style: "light" | "primary" = "light";

	const maybeBold = style == "primary" ? "-bold" : "";
	const iconName =
		modifierType === "add" ? "ph:plus" + maybeBold : "ph:minus" + maybeBold;

	let selectedDuration = {
		display: "1m",
		ms: 60000,
	};

	let buttonElem: HTMLButtonElement;
	let tippyInstance: Instance<Props>;
	const clickEvent = createEventDispatcher();
	const clickEventName = "submitupdate";

	function setUpdateAmount(event: MouseEvent) {
		const target = event.currentTarget as HTMLButtonElement;
		selectedDuration.display = target.innerHTML;
		selectedDuration.ms = +(target.dataset.time ?? 0);
		tippyInstance.hide();
	}

	function dispatchUpdateAmount() {
		clickEvent(clickEventName, { duration: selectedDuration.ms });
	}

	onMount(() => {
		tippyInstance = tippy(buttonElem, {
			content: (ref) =>
				ref.nextElementSibling ?? "Error: Something went very wrong...",
			allowHTML: true,
			interactive: true,
			appendTo: (parent) => parent.closest(".c-timer-box") ?? document.body,
			trigger: "contextmenu",
			offset: [0, 8],
			arrow: roundArrow,
			theme: "overlay-menu",
		});
	});

	onDestroy(() => {
		tippyInstance.destroy();
	});
</script>

<div class="modifier-wrapper">
	<button
		class={`duration-modifier m-${style}`}
		bind:this={buttonElem}
		on:contextmenu|preventDefault
		on:click={dispatchUpdateAmount}
	>
		<iconify-icon inline icon={iconName} />
		<span class="amount">
			{selectedDuration.display}
		</span>
	</button>
	<div class="duration-menu">
		<button
			class="wk-1"
			data-time={convert.daysToMs(7)}
			on:click={setUpdateAmount}
		>
			1w
		</button>
		<button
			class="day-1"
			data-time={convert.daysToMs(1)}
			on:click={setUpdateAmount}
		>
			1d
		</button>
		<button
			class="hr-1"
			data-time={convert.hoursToMs(1)}
			on:click={setUpdateAmount}
		>
			1h
		</button>
		<button
			class="min-30"
			data-time={convert.minsToMs(30)}
			on:click={setUpdateAmount}
		>
			30m
		</button>
		<button
			class="min-10"
			data-time={convert.minsToMs(10)}
			on:click={setUpdateAmount}
		>
			10m
		</button>
		<button
			class="min-1"
			data-time={convert.minsToMs(1)}
			on:click={setUpdateAmount}
		>
			1m
		</button>
		<button
			class="sec-30"
			data-time={convert.secsToMs(30)}
			on:click={setUpdateAmount}
		>
			30s
		</button>
		<button
			class="sec-10"
			data-time={convert.secsToMs(10)}
			on:click={setUpdateAmount}
		>
			10s
		</button>
	</div>
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
	.duration-menu {
		display: grid;
		gap: 0.5rem;
		place-items: center;
		grid-template-columns: repeat(4, 1fr);

		padding: 0.5rem;

		button {
			--s-height: 2rem;

			background-color: rgba(240, 240, 240, 0.705);
			color: var(--c-text--invert);

			height: var(--s-height);
			aspect-ratio: 2/1;
			border-radius: 1rem;

			font-size: var(--l-font-size--small);
			text-align: center;
		}
	}
</style>
