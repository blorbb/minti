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
		background-color: var(--c-primary);
		color: var(--c-primary-on);

		width: 5rem;
		height: 2rem;
		border-radius: 5rem;

		filter: var(--shadow-drop-2);
		transition: filter var(--t-transition);

		&:is(:hover, :focus-visible) {
			filter: var(--shadow-drop-3);
		}

		&:active {
			filter: none;
		}
	}
</style>
