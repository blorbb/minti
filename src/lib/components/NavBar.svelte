<script lang="ts">
	import { timerControllerList } from "$lib/utils/stores";
	import { pulse } from "$lib/utils/actions";
</script>

<nav>
	<div class="navbar-item">
		<button
			class="add-timer circled"
			on:click={timerControllerList.addTimer}
			use:pulse={"var(--c-tertiary)"}
		>
			<iconify-icon inline icon="ph:plus" />
		</button>
	</div>
	<div class="navbar-item">
		<button
			class="remove-all-timers circled"
			on:click={timerControllerList.removeAll}
			use:pulse={"var(--c-error)"}
		>
			<iconify-icon inline icon="ph:trash" />
		</button>
	</div>

	<!-- todo: just filler for now -->
	<div class="navbar-item">
		<button class="timer">
			<iconify-icon inline icon="ph:hourglass-high-fill" />
		</button>
	</div>
	<div class="navbar-item">
		<button class="stopwatch">
			<iconify-icon inline icon="ph:timer" />
		</button>
	</div>
	<div class="navbar-item">
		<button class="settings">
			<iconify-icon inline icon="ph:gear" />
		</button>
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

		background-color: var(--c-surface-1);
	}

	@media (min-aspect-ratio: 3/2) {
		nav {
			flex-direction: column;
		}
	}

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

	.circled {
		--s-color: var(--c-tertiary);

		color: var(--s-color);

		border: 2px solid var(--s-color);

		&:active {
			background-color: var(--s-color);
		}
	}

	.add-timer {
		--s-color: var(--c-tertiary);
	}

	.remove-all-timers {
		--s-color: var(--c-error);
	}
</style>
