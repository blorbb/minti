<script lang="ts">
	import "tippy.js/dist/svg-arrow.css";
	import "tippy.js/animations/scale.css";
	import "tippy.js/dist/tippy.css";
	import "../tippy.scss";
	import "../app.scss";
	import "normalize.css";

	import "iconify-icon";
	import NavBar from "$lib/components/NavBar.svelte";
	import { setCSSProp } from "$lib/utils/css";
	import { settings } from "$lib/utils/stores";
	import { onMount } from "svelte";

	setCSSProp(
		"--l-progress-bar--bg__border-width",
		$settings.progressBarBackgroundBorder.toString() + "px",
	);

	// show an overflow shadow
	// https://cushionapp.com/journal/overflow-shadows
	let contextElem: HTMLElement;
	let topShadowElem: HTMLDivElement;
	let bottomShadowElem: HTMLDivElement;
	let topEdge: HTMLDivElement;
	let bottomEdge: HTMLDivElement;

	onMount(() => {
		const observer = new IntersectionObserver(
			(entries) => {
				console.log(entries);

				for (const entry of entries) {
					const edge = (entry.target as HTMLElement).dataset["side"];
					const shadowToShow =
						edge === "top" ? topShadowElem : bottomShadowElem;

					if (entry.isIntersecting) {
						shadowToShow.style.opacity = "0";
					} else {
						shadowToShow.style.opacity = "1";
					}
				}
			},
			{
				root: contextElem,
			},
		);
		observer.observe(topEdge);
		observer.observe(bottomEdge);
	});
</script>

<svelte:head>
	<link rel="preconnect" href="https://rsms.me/" />
	<link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
</svelte:head>

<div class="viewport">
	<div class="context" bind:this={contextElem}>
		<div class="top-shadow scroll-shadow" bind:this={topShadowElem} />
		<main>
			<div class="intersection-edge" data-side="top" bind:this={topEdge} />
			<slot />
			<div
				class="intersection-edge"
				data-side="bottom"
				bind:this={bottomEdge}
			/>
		</main>
		<div class="bottom-shadow scroll-shadow" bind:this={bottomShadowElem} />
	</div>
	<NavBar />
</div>

<style lang="scss">
	.scroll-shadow {
		// TODO make shadow version customisable in settings
		// dark shadow
		// --s-shadow-color: hsla(0, 0%, 0%, 0.5);
		// --s-shadow-height: 1rem;
		// mix-blend-mode: multiply;

		// light shadow
		--s-shadow-color: hsla(184, 100%, 97%, 0.1);
		--s-shadow-height: 0.5rem;
		mix-blend-mode: screen;

		position: fixed;
		left: 0;
		right: 0;

		height: var(--s-shadow-height);

		transition: opacity var(--t-transition--short) linear;

		&.top-shadow {
			top: 0;
			background: linear-gradient(var(--s-shadow-color), transparent);
		}

		&.bottom-shadow {
			bottom: 0;
			background: linear-gradient(transparent, var(--s-shadow-color));
		}
	}

	main {
		position: relative;
	}

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

		isolation: isolate;

		// required to make `position: fixed;` relative to this.
		// using `position: absolute;` doesn't work with scrolling
		// https://stackoverflow.com/a/38796408
		transform: translate(0);

		> main {
			overflow-y: scroll;
			width: 100%;
			height: 100%;
			z-index: 0;
		}

		> .scroll-shadow {
			z-index: 1;
		}
	}

	@media (min-aspect-ratio: 3/2) {
		.viewport {
			flex-direction: row;
		}
	}
</style>
