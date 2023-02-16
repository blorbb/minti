<script lang="ts">
	import "../app.scss";
	import "normalize.css";
	import "iconify-icon";
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
	<div class="navbar">
		<div>a</div>
		<div>a</div>
	</div>
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
		overflow: unset;

		// required to make `position: fixed;` relative to this.
		// using `position: absolute;` doesn't work with scrolling
		// https://stackoverflow.com/a/38796408
		transform: translate(0);

		> main {
			overflow: scroll;
			width: 100%;
			height: 100%;
		}
	}

	.navbar {
		flex-grow: 0;
		flex-shrink: 0;
		flex-basis: var(--l-navbar-width);

		display: flex;
		align-items: center;
		justify-content: space-around;

		background-color: var(--c-surface-1);
	}

	@media (min-aspect-ratio: 3/2) {
		.viewport {
			flex-direction: row;
		}

		.navbar {
			flex-direction: column;
		}
	}
</style>
