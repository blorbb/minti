<script lang="ts">
	import type { TimeAbbreviations } from "$lib/utils/timer_controller";

	export let times: [TimeAbbreviations, string][];
</script>

{#each times as [unit, value]}
	<span class="time-value">
		<span class="time">{value}</span><span class="unit">{unit}</span>
	</span>
{/each}

<style lang="scss">
	.unit {
		font-size: 0.66666em;
		font-weight: 500;
	}
</style>
