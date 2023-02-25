<script lang="ts">
	import { settings } from "$lib/utils/stores";
	import { scale } from "svelte/transition";

	export let icon: string;
	let className = "";
	export { className as class };
</script>

<button
	class={`m-primary ${className}`}
	on:click
	in:scale={{ duration: $settings.buttonScaleDuration }}
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
