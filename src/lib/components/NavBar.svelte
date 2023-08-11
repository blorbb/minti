<script lang="ts">
	import { timerControllerList } from "$lib/utils/stores";
</script>

<nav>
	<div class="navbar-item">
		<button class="add-timer circled" on:click={timerControllerList.addTimer}>
			<iconify-icon inline icon="ph:plus" />
		</button>
	</div>
	<div class="navbar-item">
		<button
			class="remove-all-timers circled"
			on:click={timerControllerList.removeAll}
		>
			<iconify-icon inline icon="ph:trash" />
		</button>
	</div>

	<!-- todo: just filler for now -->
	<div class="navbar-item">
		<a href="/" class="timer">
			<iconify-icon inline icon="ph:hourglass-high-fill" />
		</a>
	</div>
	<div class="navbar-item">
		<button class="stopwatch">
			<iconify-icon inline icon="ph:timer" />
		</button>
	</div>
	<div class="navbar-item">
		<a href="/settings" class="settings">
			<iconify-icon inline icon="ph:gear" />
		</a>
	</div>
</nav>

<style lang="scss">
	nav {
		flex-grow: 0;
		flex-shrink: 0;
		flex-basis: var(--l-navbar__width);

		display: flex;
		align-items: center;

		> .navbar-item {
			flex-grow: 1;

			display: grid;
			place-items: center;
		}

		background-color: var(--navbar__background);
	}

	@media (min-aspect-ratio: 3/2) {
		nav {
			flex-direction: column;
		}
	}

	a,
	button {
		--s-size: calc(var(--l-navbar__width) / 1.5);

		background-color: transparent;
		color: var(--c-text);

		width: var(--s-size);
		height: var(--s-size);
		border-radius: 50%;

		&:is(:hover, :focus-visible) {
			background-color: var(--c-overlay-light);
		}
	}

	a {
		display: grid;
		place-items: center;
	}

	.circled {
		--s-color: var(--c-tertiary);

		color: var(--s-color);
		border: 2px solid var(--s-color);

		transition: background-color var(--t-transition) linear;

		&:active {
			background-color: var(--s-color);
			// only transition on release
			transition: none;
		}
	}

	.add-timer {
		--s-color: var(--c-tertiary);
	}

	.remove-all-timers {
		--s-color: var(--c-error);
	}
</style>
