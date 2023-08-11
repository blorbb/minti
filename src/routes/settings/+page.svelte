<script lang="ts">
	import Setting from "$lib/components/Setting.svelte";
	import { variableSettings } from "$lib/utils/settings";
</script>

<svelte:head>
	<title>Settings</title>
</svelte:head>

{#each variableSettings as { title, description, variableName, defaultValue }}
	<Setting {title} {description} {variableName} {defaultValue} />
{/each}
