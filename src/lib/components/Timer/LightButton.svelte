<script lang="ts">
	export let icon: string;
	let className = "";
	export { className as class };
</script>

<button class={`m-light ${className}`} on:click>
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
