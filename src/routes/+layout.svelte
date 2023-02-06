<script lang="ts">
	import "../app.scss";
	import "normalize.css";
	import "iconify-icon";
</script>

<svelte:head>
	<link rel="preconnect" href="https://rsms.me/" />
	<link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
</svelte:head>

<div class="contents">
	<div class="main-content">
		<slot />
	</div>
	<div class="navbar">
		<div>a</div>
		<div>a</div>
	</div>
</div>

<style lang="scss">
	.contents {
		display: grid;
		grid-template-rows: calc(100vh - var(--navbar-width)) var(--navbar-width);

		width: 100vw;
		height: 100vh;

		// overflow: hidden;
	}

	.main-content {
		overflow: auto;
	}

	.navbar {
		display: flex;
		align-items: center;
		justify-content: space-around;

		background-color: var(--c-surface-1);
	}

	@media (min-aspect-ratio: 3/2) {
		.contents {
			grid-template-columns: var(--navbar-width) calc(
					100vw - var(--navbar-width)
				);
			grid-template-rows: 1fr;
		}

		.navbar {
			order: -1;
			flex-direction: column;
		}
	}
</style>
