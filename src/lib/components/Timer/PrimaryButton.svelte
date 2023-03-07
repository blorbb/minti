<script lang="ts">
	import { settings } from "$lib/utils/stores";
	import { tooltip } from "$lib/utils/tippy";
	import { scale } from "svelte/transition";
	import type { Placement } from "tippy.js";

	export let icon: string;
	let className = "";
	export { className as class };
	export let tooltipContent: string | string[] = "";
	export let tooltipPlacement: Placement = "bottom";

	$: enabled = tooltipContent !== "";
</script>

<button
	class={`m-primary ${className}`}
	on:click
	in:scale={{ duration: $settings.buttonScaleDuration }}
	use:tooltip={{
		text: tooltipContent,
		theme: "translucent",
		enabled,
		tippy: { placement: tooltipPlacement },
	}}
>
	<iconify-icon inline {icon} />
</button>

<style lang="scss">
	.m-primary {
		--s-size: 2rem;
		background-color: var(--c-primary);
		color: var(--c-primary-on);

		height: var(--s-size);
		aspect-ratio: 5 / 2;
		border-radius: 5rem;

		font-size: calc(var(--s-size) / 2);

		filter: var(--shadow-2--drop);
		transition-property: filter, transform;
		transition-duration: var(--t-transition);

		&:is(:hover, :focus-visible) {
			filter: var(--shadow-3--drop);
		}

		&:active {
			filter: none;
			transform: scale(0.9);
		}
	}
</style>
