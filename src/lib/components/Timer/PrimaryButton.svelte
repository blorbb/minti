<script lang="ts">
	export let icon: string;
	let className = "";
	export { className as class };
</script>

<button class={`m-primary ${className}`} on:click>
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
