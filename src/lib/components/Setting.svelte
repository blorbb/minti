<script lang="ts">
	import { updateStyleSheet } from "$lib/utils/settings";

	export let title: string;
	export let description = "";
	export let variableName: string;
	export let defaultValue: string;
	let inputValue =
		localStorage.getItem(`setting.style.${variableName}`) ?? defaultValue;

	function update(key: string, value: string) {
		localStorage.setItem(`setting.style.${key}`, value);
		updateStyleSheet();
	}
</script>

<div class="c-setting">
	<h4>{title}</h4>
	{#if description}
		<div class="description">{description}</div>
	{/if}
	<input
		type="text"
		on:change={() => update(variableName, inputValue)}
		bind:value={inputValue}
	/>
</div>

<style lang="scss">
	.c-setting {
		background-color: #222;
		padding: 1rem;
		border-block: 1px solid var(--c-outline);
	}

	h4 {
		margin: 0;
	}

	input {
		border: 1px solid white;
		background-color: var(--c-overlay-light);
		width: 100%;
	}
</style>
