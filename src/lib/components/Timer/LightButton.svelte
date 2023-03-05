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
	class={`m-light ${className}`}
	on:click
	in:scale={{ duration: $settings.buttonScaleDuration }}
	use:tooltip={{
		text: tooltipContent,
		theme: "translucent",
		enabled,
		tippy: {
			placement: tooltipPlacement,
		},
	}}
>
	<iconify-icon inline {icon} />
</button>

<style lang="scss">
	.m-light {
		--s-size: 2rem;
		background-color: transparent;
		color: var(--c-text);

		width: var(--s-size);
		height: var(--s-size);
		border-radius: 50%;

		font-size: calc(var(--s-size) / 2);

		transition: background-color var(--t-transition);

		&:is(:hover, :focus-visible) {
			background-color: var(--c-overlay-light);
		}

		&:active {
			background-color: var(--c-overlay-lightest);
			transition: none;
		}
	}
</style>
