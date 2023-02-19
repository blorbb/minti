<script lang="ts">
	import "../app.scss";
	import "normalize.css";
	import "iconify-icon";
	import NavBar from "$lib/components/NavBar.svelte";
	import { setCSSProp } from "$lib/utils/css";
	import { settings } from "$lib/utils/stores";

	setCSSProp(
		"--l-progress-bar--bg__border-width",
		$settings.progressBarBackgroundBorder.toString() + "px",
	);
</script>

<svelte:head>
	<link rel="preconnect" href="https://rsms.me/" />
	<link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
</svelte:head>

<div class="viewport">
	<div class="context">
		<main>
			<slot />
		</main>
	</div>
	<NavBar />
</div>

<style lang="scss">
	.viewport {
		display: flex;
		flex-direction: column;

		width: 100vw;
		height: 100vh;
	}

	.context {
		flex-grow: 1;
		position: relative;
		overflow: hidden;

		// required to make `position: fixed;` relative to this.
		// using `position: absolute;` doesn't work with scrolling
		// https://stackoverflow.com/a/38796408
		transform: translate(0);

		> main {
			overflow-y: scroll;
			width: 100%;
			height: 100%;
		}
	}

	@media (min-aspect-ratio: 3/2) {
		.viewport {
			flex-direction: row;
		}
	}
</style>
